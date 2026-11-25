#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Goal {
    pub name: String,
    pub target_amount: i128,
    pub saved_amount: i128,
    pub withdrawn: bool,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Goal(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    GoalAlreadyExists = 1,
    GoalNotFound = 2,
    InvalidAmount = 3,
    GoalAlreadyWithdrawn = 4,
    GoalNotReached = 5,
    AmountOverflow = 6,
}

#[contract]
pub struct SavingGoalContract;

#[contractimpl]
impl SavingGoalContract {
    // Tạo mục tiêu tiết kiệm mới với tên và số tiền cần đạt
    pub fn create_goal(
        env: Env,
        user: Address,
        name: String,
        target_amount: i128,
    ) -> Result<Goal, Error> {
        user.require_auth();

        if target_amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let key = DataKey::Goal(user);
        if let Some(existing_goal) = env.storage().persistent().get::<DataKey, Goal>(&key) {
            if !existing_goal.withdrawn {
                return Err(Error::GoalAlreadyExists);
            }
        }

        let goal = Goal {
            name,
            target_amount,
            saved_amount: 0,
            withdrawn: false,
            created_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&key, &goal);
        Ok(goal)
    }

    // Gửi thêm tiền vào mục tiêu tiết kiệm đang mở
    pub fn deposit(env: Env, user: Address, amount: i128) -> Result<Goal, Error> {
        user.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let key = DataKey::Goal(user);
        let mut goal = env
            .storage()
            .persistent()
            .get::<DataKey, Goal>(&key)
            .ok_or(Error::GoalNotFound)?;

        if goal.withdrawn {
            return Err(Error::GoalAlreadyWithdrawn);
        }

        goal.saved_amount = goal
            .saved_amount
            .checked_add(amount)
            .ok_or(Error::AmountOverflow)?;

        env.storage().persistent().set(&key, &goal);
        Ok(goal)
    }

    // Rút toàn bộ số tiền đã tiết kiệm khi đạt đủ mục tiêu
    pub fn withdraw(env: Env, user: Address) -> Result<i128, Error> {
        user.require_auth();

        let key = DataKey::Goal(user);
        let mut goal = env
            .storage()
            .persistent()
            .get::<DataKey, Goal>(&key)
            .ok_or(Error::GoalNotFound)?;

        if goal.withdrawn {
            return Err(Error::GoalAlreadyWithdrawn);
        }

        if goal.saved_amount < goal.target_amount {
            return Err(Error::GoalNotReached);
        }

        goal.withdrawn = true;
        let total = goal.saved_amount;

        env.storage().persistent().set(&key, &goal);
        Ok(total)
    }

    // Xem thông tin mục tiêu và số tiền đã tiết kiệm của một người dùng
    pub fn get_goal(env: Env, user: Address) -> Result<Goal, Error> {
        let key = DataKey::Goal(user);
        env.storage()
            .persistent()
            .get::<DataKey, Goal>(&key)
            .ok_or(Error::GoalNotFound)
    }
}