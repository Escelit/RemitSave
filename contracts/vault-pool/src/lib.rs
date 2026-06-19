#![no_std]
use rs_shared::{DataKey, VaultError, VaultPool, YieldSource, SHARE_PRICE_DENOM, YEAR_SECS};
use soroban_sdk::{contract, contractimpl, token, Address, Env, Symbol};

#[contract]
pub struct VaultPoolContract;

#[contractimpl]
impl VaultPoolContract {
    pub fn init(
        env: Env,
        admin: Address,
        local_asset: Address,
        name: Symbol,
        yield_source: YieldSource,
        apy: u32,
        min_lockup: u64,
    ) -> Result<u32, VaultError> {
        if apy > 10000 {
            return Err(VaultError::InvalidBps);
        }

        let count_key = DataKey::GlobalPoolCount;
        let pool_id: u32 = env.storage().instance().get(&count_key).unwrap_or(0);
        env.storage().instance().set(&count_key, &(pool_id + 1));

        let now = env.ledger().timestamp();
        let pool = VaultPool {
            pool_id,
            local_asset,
            name,
            total_deposits: 0,
            total_shares: 0,
            share_price: SHARE_PRICE_DENOM,
            yield_source,
            apy,
            admin,
            min_lockup,
            created_at: now,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Pool(pool_id), &pool);
        env.storage()
            .persistent()
            .set(&DataKey::PoolLastUpdated(pool_id), &now);

        Ok(pool_id)
    }

    pub fn vault_deposit(
        env: Env,
        pool_id: u32,
        user: Address,
        amount: i128,
    ) -> Result<i128, VaultError> {
        user.require_auth();
        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        Self::accrue_yield(&env, pool_id)?;

        let pool_key = DataKey::Pool(pool_id);
        let mut pool: VaultPool = env
            .storage()
            .persistent()
            .get(&pool_key)
            .ok_or(VaultError::PoolNotFound)?;

        let shares = amount
            .checked_mul(SHARE_PRICE_DENOM)
            .ok_or(VaultError::Overflow)?
            .checked_div(pool.share_price)
            .ok_or(VaultError::Overflow)?;

        if shares <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        let token_client = token::Client::new(&env, &pool.local_asset);
        token_client.transfer(&user, &env.current_contract_address(), &amount);

        pool.total_deposits = pool
            .total_deposits
            .checked_add(amount)
            .ok_or(VaultError::Overflow)?;
        pool.total_shares = pool
            .total_shares
            .checked_add(shares)
            .ok_or(VaultError::Overflow)?;

        env.storage().persistent().set(&pool_key, &pool);

        let user_key = DataKey::PoolUserBalance(user.clone(), pool_id);
        let existing: i128 = env.storage().persistent().get(&user_key).unwrap_or(0);
        env.storage().persistent().set(
            &user_key,
            &existing.checked_add(shares).ok_or(VaultError::Overflow)?,
        );

        let lock_key = DataKey::PoolUserDepositTime(user, pool_id);
        env.storage()
            .persistent()
            .set(&lock_key, &env.ledger().timestamp());

        Ok(shares)
    }

    pub fn vault_withdraw(
        env: Env,
        pool_id: u32,
        user: Address,
        share_amount: i128,
    ) -> Result<i128, VaultError> {
        user.require_auth();
        if share_amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        Self::accrue_yield(&env, pool_id)?;

        let pool_key = DataKey::Pool(pool_id);
        let mut pool: VaultPool = env
            .storage()
            .persistent()
            .get(&pool_key)
            .ok_or(VaultError::PoolNotFound)?;

        let user_key = DataKey::PoolUserBalance(user.clone(), pool_id);
        let user_shares: i128 = env.storage().persistent().get(&user_key).unwrap_or(0);

        if share_amount > user_shares {
            return Err(VaultError::InsufficientShares);
        }

        let lock_key = DataKey::PoolUserDepositTime(user.clone(), pool_id);
        let deposit_time: u64 = env.storage().persistent().get(&lock_key).unwrap_or(0);
        let elapsed = env.ledger().timestamp().wrapping_sub(deposit_time);
        if elapsed < pool.min_lockup {
            return Err(VaultError::LockupActive);
        }

        let amount = share_amount
            .checked_mul(pool.share_price)
            .ok_or(VaultError::Overflow)?
            .checked_div(SHARE_PRICE_DENOM)
            .ok_or(VaultError::Overflow)?;

        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }
        if amount > pool.total_deposits {
            return Err(VaultError::InsufficientLiquidity);
        }

        let token_client = token::Client::new(&env, &pool.local_asset);
        token_client.transfer(&env.current_contract_address(), &user, &amount);

        pool.total_deposits = pool
            .total_deposits
            .checked_sub(amount)
            .ok_or(VaultError::Overflow)?;
        pool.total_shares = pool
            .total_shares
            .checked_sub(share_amount)
            .ok_or(VaultError::Overflow)?;

        env.storage().persistent().set(&pool_key, &pool);

        let remaining = user_shares - share_amount;
        if remaining > 0 {
            env.storage().persistent().set(&user_key, &remaining);
        } else {
            env.storage().persistent().remove(&user_key);
        }

        Ok(amount)
    }

    pub fn accrue_yield(env: &Env, pool_id: u32) -> Result<(), VaultError> {
        let pool_key = DataKey::Pool(pool_id);
        let mut pool: VaultPool = env
            .storage()
            .persistent()
            .get(&pool_key)
            .ok_or(VaultError::PoolNotFound)?;

        let last_key = DataKey::PoolLastUpdated(pool_id);
        let last_updated: u64 = env
            .storage()
            .persistent()
            .get(&last_key)
            .unwrap_or(pool.created_at);

        let now = env.ledger().timestamp();
        if now <= last_updated || pool.total_shares == 0 {
            return Ok(());
        }

        let elapsed = now - last_updated;

        let yield_earned = pool
            .total_deposits
            .checked_mul(pool.apy as i128)
            .ok_or(VaultError::Overflow)?
            .checked_mul(elapsed as i128)
            .ok_or(VaultError::Overflow)?
            .checked_div(YEAR_SECS as i128)
            .ok_or(VaultError::Overflow)?
            .checked_div(10000)
            .ok_or(VaultError::Overflow)?;

        if yield_earned > 0 {
            pool.total_deposits = pool
                .total_deposits
                .checked_add(yield_earned)
                .ok_or(VaultError::Overflow)?;
        }

        pool.share_price = pool
            .total_deposits
            .checked_mul(SHARE_PRICE_DENOM)
            .ok_or(VaultError::Overflow)?
            .checked_div(pool.total_shares)
            .ok_or(VaultError::Overflow)?;

        env.storage().persistent().set(&pool_key, &pool);
        env.storage().persistent().set(&last_key, &now);

        Ok(())
    }

    pub fn update_share_price(env: Env, pool_id: u32) -> Result<(), VaultError> {
        Self::accrue_yield(&env, pool_id)
    }

    pub fn get_pool_info(env: Env, pool_id: u32) -> Result<VaultPool, VaultError> {
        env.storage()
            .persistent()
            .get(&DataKey::Pool(pool_id))
            .ok_or(VaultError::PoolNotFound)
    }

    pub fn user_share_balance(env: Env, user: Address, pool_id: u32) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::PoolUserBalance(user, pool_id))
            .unwrap_or(0)
    }

    pub fn set_apy(env: Env, pool_id: u32, admin: Address, new_apy: u32) -> Result<(), VaultError> {
        admin.require_auth();
        if new_apy > 10000 {
            return Err(VaultError::InvalidBps);
        }

        Self::accrue_yield(&env, pool_id)?;

        let pool_key = DataKey::Pool(pool_id);
        let mut pool: VaultPool = env
            .storage()
            .persistent()
            .get(&pool_key)
            .ok_or(VaultError::PoolNotFound)?;

        if pool.admin != admin {
            return Err(VaultError::Unauthorized);
        }

        pool.apy = new_apy;
        env.storage().persistent().set(&pool_key, &pool);

        Ok(())
    }
}

#[cfg(test)]
mod test;
