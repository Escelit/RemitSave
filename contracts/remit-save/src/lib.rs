#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Address, Symbol, Bytes, Vec};
use rs_shared::{
    UserProfile, PlanStatus, SavingsPlan, SplitType, RemittanceRule, DataKey, RemittanceResult, RemitError
};

#[contract]
pub struct RemitSave;

#[contractimpl]
impl RemitSave {
    pub fn initialize(
        env: Env,
        admin: Address,
        fee_recipient: Address,
        protocol_fee_bps: u32,
    ) -> Result<(), RemitError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(RemitError::AlreadyInitialized);
        }
        if protocol_fee_bps > 10000 {
            return Err(RemitError::InvalidBps);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::FeeRecipient, &fee_recipient);
        env.storage().instance().set(&DataKey::ProtocolFeeBps, &protocol_fee_bps);
        Ok(())
    }

    pub fn register_user(
        env: Env,
        user: Address,
        country: Symbol,
        phone: Bytes,
    ) -> Result<(), RemitError> {
        user.require_auth();
        
        let key = DataKey::User(user.clone());
        if env.storage().persistent().has(&key) {
            return Err(RemitError::AlreadyInitialized);
        }
        
        let profile = UserProfile {
            stellar_address: user.clone(),
            country,
            phone,
            kyc_level: 0,
            created_at: env.ledger().timestamp(),
            last_active: env.ledger().timestamp(),
        };
        env.storage().persistent().set(&key, &profile);
        Ok(())
    }

    pub fn get_user(env: Env, user: Address) -> Result<UserProfile, RemitError> {
        let key = DataKey::User(user);
        env.storage()
            .persistent()
            .get(&key)
            .ok_or(RemitError::UserNotFound)
    }

    pub fn create_savings_plan(
        env: Env,
        owner: Address,
        goal_name: Symbol,
        target_amount: i128,
        local_asset: Address,
        lock_until: Option<u64>,
    ) -> Result<u32, RemitError> {
        owner.require_auth();
        if target_amount <= 0 {
            return Err(RemitError::InvalidAmount);
        }
        
        let count_key = DataKey::GlobalPlanCount(owner.clone());
        let plan_id: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);
        let next_plan_id = plan_id + 1;
        env.storage().persistent().set(&count_key, &next_plan_id);
        
        let plan = SavingsPlan {
            owner: owner.clone(),
            plan_id,
            goal_name,
            target_amount,
            balance: 0,
            local_asset,
            auto_save_pct: 0,
            lock_until,
            status: PlanStatus::Active,
            created_at: env.ledger().timestamp(),
        };
        
        let plan_key = DataKey::Plan(owner, plan_id);
        env.storage().persistent().set(&plan_key, &plan);
        
        Ok(plan_id)
    }

    pub fn get_savings_plan(env: Env, owner: Address, plan_id: u32) -> Result<SavingsPlan, RemitError> {
        let key = DataKey::Plan(owner, plan_id);
        env.storage()
            .persistent()
            .get(&key)
            .ok_or(RemitError::PlanNotFound)
    }

    pub fn list_savings_plans(env: Env, owner: Address) -> Vec<SavingsPlan> {
        let count_key = DataKey::GlobalPlanCount(owner.clone());
        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);
        let mut plans = Vec::new(&env);
        for id in 0..count {
            let key = DataKey::Plan(owner.clone(), id);
            if let Some(plan) = env.storage().persistent().get::<_, SavingsPlan>(&key) {
                plans.push_back(plan);
            }
        }
        plans
    }

    pub fn set_remittance_rule(
        env: Env,
        sender: Address,
        rule: RemittanceRule,
    ) -> Result<u32, RemitError> {
        sender.require_auth();
        
        if rule.sender != sender {
            return Err(RemitError::Unauthorized);
        }
        
        if let SplitType::Percentage = rule.split_type {
            if rule.split_value > 10000 {
                return Err(RemitError::InvalidBps);
            }
        }
        
        if let Some(plan_id) = rule.savings_plan_id {
            let plan_key = DataKey::Plan(sender.clone(), plan_id);
            let mut plan: SavingsPlan = env.storage()
                .persistent()
                .get(&plan_key)
                .ok_or(RemitError::PlanNotFound)?;
            if let PlanStatus::Active = plan.status {
                if let SplitType::Percentage = rule.split_type {
                    plan.auto_save_pct = rule.split_value;
                    env.storage().persistent().set(&plan_key, &plan);
                }
            } else {
                return Err(RemitError::PlanClosed);
            }
        }
        
        let count_key = DataKey::GlobalRuleCount(sender.clone());
        let rule_id: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);
        let next_rule_id = rule_id + 1;
        env.storage().persistent().set(&count_key, &next_rule_id);
        
        let rule_key = DataKey::Rule(sender, rule_id);
        env.storage().persistent().set(&rule_key, &rule);
        
        Ok(rule_id)
    }

    pub fn get_remittance_rule(
        env: Env,
        sender: Address,
        rule_id: u32,
    ) -> Result<RemittanceRule, RemitError> {
        let key = DataKey::Rule(sender, rule_id);
        env.storage()
            .persistent()
            .get(&key)
            .ok_or(RemitError::RuleNotFound)
    }

    pub fn list_remittance_rules(env: Env, sender: Address) -> Vec<RemittanceRule> {
        let count_key = DataKey::GlobalRuleCount(sender.clone());
        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);
        let mut rules = Vec::new(&env);
        for id in 0..count {
            let key = DataKey::Rule(sender.clone(), id);
            if let Some(rule) = env.storage().persistent().get::<_, RemittanceRule>(&key) {
                rules.push_back(rule);
            }
        }
        rules
    }

    pub fn deactivate_remittance_rule(
        env: Env,
        sender: Address,
        rule_id: u32,
    ) -> Result<(), RemitError> {
        sender.require_auth();
        let key = DataKey::Rule(sender.clone(), rule_id);
        let mut rule: RemittanceRule = env.storage()
            .persistent()
            .get(&key)
            .ok_or(RemitError::RuleNotFound)?;
        rule.active = false;
        env.storage().persistent().set(&key, &rule);
        Ok(())
    }

    pub fn set_anchor(
        env: Env,
        asset: Address,
        anchor_address: Address,
    ) -> Result<(), RemitError> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).ok_or(RemitError::NotInitialized)?;
        admin.require_auth();
        
        env.storage().instance().set(&DataKey::Anchor(asset), &anchor_address);
        Ok(())
    }

    pub fn execute_remittance(
        env: Env,
        sender: Address,
        rule_id: u32,
        total_amount: i128,
        incoming_asset: Address,
    ) -> Result<RemittanceResult, RemitError> {
        sender.require_auth();
        
        if total_amount <= 0 {
            return Err(RemitError::InvalidAmount);
        }
        
        let rule_key = DataKey::Rule(sender.clone(), rule_id);
        let rule: RemittanceRule = env.storage()
            .persistent()
            .get(&rule_key)
            .ok_or(RemitError::RuleNotFound)?;
            
        if !rule.active {
            return Err(RemitError::RuleNotFound);
        }
        
        if rule.incoming_asset != incoming_asset {
            return Err(RemitError::Unauthorized);
        }
        
        let _admin: Address = env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(RemitError::NotInitialized)?;
        let fee_recipient: Address = env.storage()
            .instance()
            .get(&DataKey::FeeRecipient)
            .ok_or(RemitError::NotInitialized)?;
        let fee_bps: u32 = env.storage()
            .instance()
            .get(&DataKey::ProtocolFeeBps)
            .ok_or(RemitError::NotInitialized)?;
            
        let fee_amount = total_amount
            .checked_mul(fee_bps as i128)
            .ok_or(RemitError::Overflow)?
            .checked_div(10000)
            .ok_or(RemitError::Overflow)?;
            
        let net_amount = total_amount
            .checked_sub(fee_amount)
            .ok_or(RemitError::Overflow)?;
            
        if net_amount <= 0 {
            return Err(RemitError::InvalidAmount);
        }
        
        let savings_amount_incoming = if rule.savings_plan_id.is_some() {
            match rule.split_type {
                SplitType::Percentage => {
                    net_amount
                        .checked_mul(rule.split_value as i128)
                        .ok_or(RemitError::Overflow)?
                        .checked_div(10000)
                        .ok_or(RemitError::Overflow)?
                }
                SplitType::Fixed => {
                    let val = rule.split_value as i128;
                    if val > net_amount {
                        net_amount
                    } else {
                        val
                    }
                }
            }
        } else {
            0
        };
        
        let payout_amount_incoming = net_amount
            .checked_sub(savings_amount_incoming)
            .ok_or(RemitError::Overflow)?;
            
        let token_client = soroban_sdk::token::Client::new(&env, &incoming_asset);
        
        // --- Execute Transfers ---
        
        // 1. Collect Fees
        if fee_amount > 0 {
            token_client.transfer(&sender, &fee_recipient, &fee_amount);
        }
        
        // 2. Execute Payout (Mocking Path Payment)
        // In a real scenario, this would be a path_payment call.
        // For the mock, we transfer USDC to the anchor who disburses local currency.
        let anchor: Address = env.storage()
            .instance()
            .get(&DataKey::Anchor(rule.local_asset.clone()))
            .unwrap_or(rule.beneficiary.clone()); // Fallback to beneficiary for testing
            
        if payout_amount_incoming > 0 {
            token_client.transfer(&sender, &anchor, &payout_amount_incoming);
        }
        
        // 3. Execute Savings (Mocking Path Payment)
        // We transfer USDC to the contract (which acts as the vault/escrow)
        let mut savings_amount_local = 0;
        if savings_amount_incoming > 0 {
            if let Some(plan_id) = rule.savings_plan_id {
                let plan_key = DataKey::Plan(sender.clone(), plan_id);
                let mut plan: SavingsPlan = env.storage()
                    .persistent()
                    .get(&plan_key)
                    .ok_or(RemitError::PlanNotFound)?;
                    
                if let PlanStatus::Active = plan.status {
                    token_client.transfer(&sender, &env.current_contract_address(), &savings_amount_incoming);
                    
                    // In a real scenario, savings_amount_local would be the output of a DEX path payment.
                    // For this mock, we assume 1:1 conversion to local asset units.
                    savings_amount_local = savings_amount_incoming;
                        
                    plan.balance = plan.balance
                        .checked_add(savings_amount_local)
                        .ok_or(RemitError::Overflow)?;
                    env.storage().persistent().set(&plan_key, &plan);
                } else {
                    return Err(RemitError::PlanClosed);
                }
            }
        }

        // Mock conversion for payout event as well
        let payout_amount_local = payout_amount_incoming;

        // --- Emit Detailed Event ---
        let rem_count_key = DataKey::GlobalRemittanceCount;
        let rem_id: u32 = env.storage().instance().get(&rem_count_key).unwrap_or(0);
        env.storage().instance().set(&rem_count_key, &(rem_id + 1));

        use rs_shared::RemittanceExecuted;
        let event = RemittanceExecuted {
            remittance_id: rem_id,
            sender: sender.clone(),
            beneficiary: rule.beneficiary.clone(),
            total_amount,
            payout_amount: payout_amount_local,
            savings_amount: savings_amount_local,
            fee_amount,
            incoming_asset: incoming_asset.clone(),
            local_asset: rule.local_asset.clone(),
            timestamp: env.ledger().timestamp(),
        };
        
        env.events().publish(
            (Symbol::new(&env, "remittance_executed"), sender, rule.beneficiary),
            event
        );
        
        Ok(RemittanceResult {
            payout_amount: payout_amount_local,
            savings_amount: savings_amount_local,
            fee_amount,
        })
    }

    pub fn deposit_to_plan(
        env: Env,
        owner: Address,
        plan_id: u32,
        amount: i128,
        asset: Address,
    ) -> Result<(), RemitError> {
        owner.require_auth();
        if amount <= 0 {
            return Err(RemitError::InvalidAmount);
        }
        
        let plan_key = DataKey::Plan(owner.clone(), plan_id);
        let mut plan: SavingsPlan = env.storage()
            .persistent()
            .get(&plan_key)
            .ok_or(RemitError::PlanNotFound)?;
            
        if let PlanStatus::Active = plan.status {
            if plan.local_asset != asset {
                return Err(RemitError::Unauthorized);
            }
            
            let token_client = soroban_sdk::token::Client::new(&env, &asset);
            token_client.transfer(&owner, &env.current_contract_address(), &amount);
            
            plan.balance = plan.balance
                .checked_add(amount)
                .ok_or(RemitError::Overflow)?;
            env.storage().persistent().set(&plan_key, &plan);
            Ok(())
        } else {
            Err(RemitError::PlanClosed)
        }
    }

    pub fn withdraw_from_plan(
        env: Env,
        owner: Address,
        plan_id: u32,
    ) -> Result<i128, RemitError> {
        owner.require_auth();
        let plan_key = DataKey::Plan(owner.clone(), plan_id);
        let mut plan: SavingsPlan = env.storage()
            .persistent()
            .get(&plan_key)
            .ok_or(RemitError::PlanNotFound)?;
            
        if let PlanStatus::Active = plan.status {
            if let Some(lock_until) = plan.lock_until {
                if env.ledger().timestamp() < lock_until {
                    return Err(RemitError::Unauthorized);
                }
            }
            
            let withdraw_amount = plan.balance;
            if withdraw_amount <= 0 {
                return Err(RemitError::InvalidAmount);
            }
            
            let token_client = soroban_sdk::token::Client::new(&env, &plan.local_asset);
            token_client.transfer(&env.current_contract_address(), &owner, &withdraw_amount);
            
            plan.balance = 0;
            plan.status = PlanStatus::Withdrawn;
            env.storage().persistent().set(&plan_key, &plan);
            
            Ok(withdraw_amount)
        } else {
            Err(RemitError::PlanClosed)
        }
    }

    pub fn close_savings_plan(
        env: Env,
        owner: Address,
        plan_id: u32,
    ) -> Result<i128, RemitError> {
        owner.require_auth();
        let plan_key = DataKey::Plan(owner.clone(), plan_id);
        let mut plan: SavingsPlan = env.storage()
            .persistent()
            .get(&plan_key)
            .ok_or(RemitError::PlanNotFound)?;
            
        if let PlanStatus::Closed = plan.status {
            return Err(RemitError::PlanClosed);
        }
        
        if let Some(lock_until) = plan.lock_until {
            if env.ledger().timestamp() < lock_until {
                return Err(RemitError::Unauthorized);
            }
        }
        
        let withdraw_amount = plan.balance;
        if withdraw_amount > 0 {
            let token_client = soroban_sdk::token::Client::new(&env, &plan.local_asset);
            token_client.transfer(&env.current_contract_address(), &owner, &withdraw_amount);
        }
        
        plan.balance = 0;
        plan.status = PlanStatus::Closed;
        env.storage().persistent().set(&plan_key, &plan);
        
        Ok(withdraw_amount)
    }
}

#[cfg(test)]
mod test;
