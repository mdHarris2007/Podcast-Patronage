#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, Env, Map, String, Symbol, Vec,
};

// ─────────────────────────────────────────────
//  Storage key types
// ─────────────────────────────────────────────
#[contracttype]
pub enum DataKey {
    Podcast(Address),        // podcaster address → PodcastInfo
    Subscription(Address, Address), // (listener, podcaster) → SubscriptionInfo
    Listeners(Address),      // podcaster → Vec<Address>
}

// ─────────────────────────────────────────────
//  Data structures
// ─────────────────────────────────────────────
#[contracttype]
#[derive(Clone)]
pub struct PodcastInfo {
    pub owner: Address,
    pub name: String,
    pub price_per_period: i128, // stroops (1 XLM = 10_000_000 stroops)
    pub period_seconds: u64,    // e.g. 2_592_000 = 30 days
    pub token: Address,         // SAC token used for payments
    pub total_collected: i128,
    pub active: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct SubscriptionInfo {
    pub listener: Address,
    pub podcaster: Address,
    pub started_at: u64,
    pub last_charged_at: u64,
    pub periods_paid: u32,
    pub active: bool,
}

// ─────────────────────────────────────────────
//  Events (topics emitted so indexers can track)
// ─────────────────────────────────────────────
const TOPIC_REGISTERED: &str = "podcast_registered";
const TOPIC_SUBSCRIBED: &str = "subscribed";
const TOPIC_PAYMENT: &str = "payment_collected";
const TOPIC_UNSUBSCRIBED: &str = "unsubscribed";

// ─────────────────────────────────────────────
//  Contract
// ─────────────────────────────────────────────
#[contract]
pub struct PodcastFundContract;

#[contractimpl]
impl PodcastFundContract {
    // ── Podcaster: register a podcast ──────────────────────────────────────
    pub fn register_podcast(
        env: Env,
        owner: Address,
        name: String,
        price_per_period: i128,
        period_seconds: u64,
        token: Address,
    ) -> PodcastInfo {
        owner.require_auth();

        assert!(price_per_period > 0, "price must be > 0");
        assert!(period_seconds > 0, "period must be > 0");

        let info = PodcastInfo {
            owner: owner.clone(),
            name: name.clone(),
            price_per_period,
            period_seconds,
            token,
            total_collected: 0,
            active: true,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Podcast(owner.clone()), &info);

        env.events().publish(
            (Symbol::new(&env, TOPIC_REGISTERED), owner),
            name,
        );

        info
    }

    // ── Listener: subscribe & pay first period immediately ─────────────────
    pub fn subscribe(env: Env, listener: Address, podcaster: Address) -> SubscriptionInfo {
        listener.require_auth();

        let mut podcast: PodcastInfo = env
            .storage()
            .persistent()
            .get(&DataKey::Podcast(podcaster.clone()))
            .expect("podcast not found");

        assert!(podcast.active, "podcast is not active");

        let key = DataKey::Subscription(listener.clone(), podcaster.clone());
        assert!(
            !env.storage().persistent().has(&key),
            "already subscribed"
        );

        // Transfer first period payment from listener → podcaster
        let token_client = token::Client::new(&env, &podcast.token);
        token_client.transfer(&listener, &podcaster, &podcast.price_per_period);

        let now = env.ledger().timestamp();
        let sub = SubscriptionInfo {
            listener: listener.clone(),
            podcaster: podcaster.clone(),
            started_at: now,
            last_charged_at: now,
            periods_paid: 1,
            active: true,
        };

        env.storage().persistent().set(&key, &sub);

        // Track listener in podcaster's list
        let listeners_key = DataKey::Listeners(podcaster.clone());
        let mut listeners: Vec<Address> = env
            .storage()
            .persistent()
            .get(&listeners_key)
            .unwrap_or(Vec::new(&env));
        listeners.push_back(listener.clone());
        env.storage().persistent().set(&listeners_key, &listeners);

        // Update podcast totals
        podcast.total_collected = podcast
            .total_collected
            .checked_add(podcast.price_per_period)
            .expect("overflow");
        env.storage()
            .persistent()
            .set(&DataKey::Podcast(podcaster.clone()), &podcast);

        env.events().publish(
            (Symbol::new(&env, TOPIC_SUBSCRIBED), listener, podcaster),
            now,
        );

        sub
    }

    // ── Anyone: trigger a recurring charge for a listener ─────────────────
    //   Can be called by a keeper/bot each period.
    pub fn collect_payment(env: Env, listener: Address, podcaster: Address) -> i128 {
        let key = DataKey::Subscription(listener.clone(), podcaster.clone());
        let mut sub: SubscriptionInfo = env
            .storage()
            .persistent()
            .get(&key)
            .expect("subscription not found");

        assert!(sub.active, "subscription is not active");

        let mut podcast: PodcastInfo = env
            .storage()
            .persistent()
            .get(&DataKey::Podcast(podcaster.clone()))
            .expect("podcast not found");

        let now = env.ledger().timestamp();
        let elapsed = now.saturating_sub(sub.last_charged_at);
        assert!(
            elapsed >= podcast.period_seconds,
            "payment period not elapsed"
        );

        // Transfer recurring payment
        let token_client = token::Client::new(&env, &podcast.token);
        token_client.transfer(&listener, &podcaster, &podcast.price_per_period);

        sub.last_charged_at = now;
        sub.periods_paid = sub.periods_paid.saturating_add(1);
        env.storage().persistent().set(&key, &sub);

        podcast.total_collected = podcast
            .total_collected
            .checked_add(podcast.price_per_period)
            .expect("overflow");
        env.storage()
            .persistent()
            .set(&DataKey::Podcast(podcaster.clone()), &podcast);

        env.events().publish(
            (Symbol::new(&env, TOPIC_PAYMENT), listener, podcaster),
            podcast.price_per_period,
        );

        podcast.price_per_period
    }

    // ── Listener: cancel subscription ─────────────────────────────────────
    pub fn unsubscribe(env: Env, listener: Address, podcaster: Address) {
        listener.require_auth();

        let key = DataKey::Subscription(listener.clone(), podcaster.clone());
        let mut sub: SubscriptionInfo = env
            .storage()
            .persistent()
            .get(&key)
            .expect("subscription not found");

        sub.active = false;
        env.storage().persistent().set(&key, &sub);

        env.events().publish(
            (Symbol::new(&env, TOPIC_UNSUBSCRIBED), listener, podcaster),
            env.ledger().timestamp(),
        );
    }

    // ── Podcaster: deactivate their podcast ────────────────────────────────
    pub fn deactivate_podcast(env: Env, owner: Address) {
        owner.require_auth();

        let mut podcast: PodcastInfo = env
            .storage()
            .persistent()
            .get(&DataKey::Podcast(owner.clone()))
            .expect("podcast not found");

        podcast.active = false;
        env.storage()
            .persistent()
            .set(&DataKey::Podcast(owner.clone()), &podcast);
    }

    // ── Read helpers ───────────────────────────────────────────────────────
    pub fn get_podcast(env: Env, podcaster: Address) -> PodcastInfo {
        env.storage()
            .persistent()
            .get(&DataKey::Podcast(podcaster))
            .expect("podcast not found")
    }

    pub fn get_subscription(
        env: Env,
        listener: Address,
        podcaster: Address,
    ) -> SubscriptionInfo {
        env.storage()
            .persistent()
            .get(&DataKey::Subscription(listener, podcaster))
            .expect("subscription not found")
    }

    pub fn get_listeners(env: Env, podcaster: Address) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::Listeners(podcaster))
            .unwrap_or(Vec::new(&env))
    }

    pub fn is_subscribed(env: Env, listener: Address, podcaster: Address) -> bool {
        let key = DataKey::Subscription(listener, podcaster);
        if let Some(sub) = env
            .storage()
            .persistent()
            .get::<DataKey, SubscriptionInfo>(&key)
        {
            sub.active
        } else {
            false
        }
    }
}