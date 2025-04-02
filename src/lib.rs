#![no_std]

multiversx_sc::imports!();

pub mod common;
pub mod helpers;
pub mod swap;
pub mod liquidity;

use common::{config::*, errors::*};
use tfn_platform::common::config::ProxyTrait as _;
use tfn_dex::common::{errors::*, consts::*};

#[multiversx_sc::contract]
pub trait TFNTestDEXContract<ContractReader>:
common::config::ConfigModule
+helpers::HelpersModule
+swap::SwapModule
+liquidity::LiquidityModule
{
    #[init]
    fn init(&self, platform_sc: ManagedAddress) {
        self.platform_sc().set(platform_sc);
        let governance_token = self.platform_contract_proxy()
            .contract(self.platform_sc().get())
            .governance_token()
            .execute_on_dest_context::<TokenIdentifier>();
        self.base_tokens().insert(governance_token);
        self.set_state_active();
    }

    #[upgrade]
    fn upgrade(&self) {
    }

    #[payable("EGLD")]
    #[endpoint(createPair)]
    fn create_pair(
        &self,
        base_token: TokenIdentifier,
        token: TokenIdentifier,
        decimals: u8,
        lp_fee: u64,
        owner_fee: u64,
    ) {
        let caller = self.blockchain().get_caller();
        self.check_whitelisted(&caller);
        require!(self.state().get() == State::Active, ERROR_NOT_ACTIVE);
        require!(self.base_tokens().contains(&base_token), ERROR_WRONG_BASE_TOKEN);
        require!(base_token != token, ERROR_WRONG_BASE_TOKEN);
        require!(self.get_pair_by_tickers(&token, &base_token).is_none(), ERROR_PAIR_EXISTS);

        let mut lp_ticker = token.ticker().concat(base_token.ticker());
        let prefix_suffix_len = LP_TOKEN_PREFIX.len() + LP_TOKEN_SUFFIX.len();
        if lp_ticker.len() > 20 - prefix_suffix_len {
            lp_ticker = lp_ticker.copy_slice(0, 20 - prefix_suffix_len).unwrap();
        }
        let lp_name = ManagedBuffer::from(LP_TOKEN_PREFIX)
            .concat(lp_ticker.clone())
            .concat(ManagedBuffer::from(LP_TOKEN_SUFFIX));
        if lp_ticker.len() > 10 {
            lp_ticker = lp_ticker.copy_slice(0, 10).unwrap();
        }
        let issue_cost = self.call_value().egld_value().clone_value();

        self.send()
            .esdt_system_sc_proxy()
            .issue_fungible(
                issue_cost,
                lp_name,
                lp_ticker,
                BigUint::zero(),
                FungibleTokenProperties {
                    num_decimals: LP_TOKEN_DECIMALS,
                    can_freeze: true,
                    can_wipe: true,
                    can_pause: true,
                    can_mint: true,
                    can_burn: true,
                    can_change_owner: true,
                    can_upgrade: true,
                    can_add_special_roles: true,
                },
            )
            .with_callback(self.callbacks().lp_token_issue_callback(
                caller,
                &base_token,
                &token,
                decimals,
                lp_fee,
                owner_fee,
            ))
            .async_call_and_exit();
    }

    #[callback]
    fn lp_token_issue_callback(
        &self,
        caller: ManagedAddress,
        base_token: &TokenIdentifier,
        token: &TokenIdentifier,
        decimals: u8,
        lp_fee: u64,
        owner_fee: u64,
        #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(lp_token) => {
                let id = self.last_pair_id().get();
                let pair = Pair {
                    id,
                    owner: caller,
                    state: PairState::ActiveNoSwap,
                    token: token.clone(),
                    decimals,
                    base_token: base_token.clone(),
                    lp_token,
                    lp_supply: BigUint::zero(),
                    lp_fee,
                    owner_fee,
                    liquidity_token: BigUint::zero(),
                    liquidity_base: BigUint::zero(),
                };
                self.last_pair_id().set(id + 1);
                self.pair(id).set(pair);
            }
            ManagedAsyncCallResult::Err(_) => {
                let issue_cost = self.call_value().egld_value();
                self.send().direct_egld(&caller, &issue_cost);
            }
        }
    }

    #[endpoint(setPairActive)]
    fn set_pair_active(&self, id: usize) {
        require!(self.state().get() == State::Active, ERROR_NOT_ACTIVE);
        require!(!self.pair(id).is_empty(), ERROR_PAIR_NOT_FOUND);

        let mut pair = self.pair(id).get();
        require!(pair.owner == self.blockchain().get_caller(), ERROR_NOT_PAIR_OWNER);
        require!(pair.lp_supply > 0, ERROR_NO_LIQUIDITY);

        pair.state = PairState::Active;
        self.pair(id).set(pair);
    }

    #[endpoint(setPairActiveNoSwap)]
    fn set_pair_active_no_swap(&self, id: usize) {
        require!(self.state().get() == State::Active, ERROR_NOT_ACTIVE);
        require!(!self.pair(id).is_empty(), ERROR_PAIR_NOT_FOUND);

        let mut pair = self.pair(id).get();
        require!(pair.owner == self.blockchain().get_caller(), ERROR_NOT_PAIR_OWNER);
        require!(pair.lp_supply > 0, ERROR_NO_LIQUIDITY);

        pair.state = PairState::ActiveNoSwap;
        self.pair(id).set(pair);
    }

    #[endpoint(setPairInactive)]
    fn set_pair_inactive(&self, id: usize) {
        require!(self.state().get() == State::Active, ERROR_NOT_ACTIVE);
        require!(!self.pair(id).is_empty(), ERROR_PAIR_NOT_FOUND);

        let mut pair = self.pair(id).get();
        require!(pair.owner == self.blockchain().get_caller(), ERROR_NOT_PAIR_OWNER);

        pair.state = PairState::Inactive;
        self.pair(id).set(pair);
    }

    #[endpoint(changePairFees)]
    fn change_pair_fees(
        &self,
        id: usize,
        new_lp_fee: u64,
        new_owner_fee: u64,
    ) {
        require!(self.state().get() == State::Active, ERROR_NOT_ACTIVE);
        require!(!self.pair(id).is_empty(), ERROR_PAIR_NOT_FOUND);

        let mut pair = self.pair(id).get();
        require!(pair.owner == self.blockchain().get_caller(), ERROR_NOT_PAIR_OWNER);

        pair.lp_fee = new_lp_fee;
        pair.owner_fee = new_owner_fee;
        self.pair(id).set(pair);
    }

    #[endpoint(addBaseToken)]
    fn add_base_token(&self, token: TokenIdentifier) {
        require!(self.state().get() == State::Active, ERROR_NOT_ACTIVE);
        require!(!self.base_tokens().contains(&token), ERROR_BASE_TOKEN_EXISTS);
        self.check_whitelisted(&self.blockchain().get_caller());

        self.base_tokens().insert(token);
    }

    #[endpoint(removeBaseToken)]
    fn remove_base_token(&self, token: TokenIdentifier) {
        require!(self.state().get() == State::Active, ERROR_NOT_ACTIVE);
        require!(self.base_tokens().contains(&token), ERROR_WRONG_BASE_TOKEN);
        self.check_whitelisted(&self.blockchain().get_caller());

        for pair_id in 0..self.last_pair_id().get() {
            if self.pair(pair_id).is_empty() {
                continue;
            }

            let pair = self.pair(pair_id).get();
            require!(pair.base_token != token, ERROR_BASE_TOKEN_IN_USE);
        }
        self.base_tokens().swap_remove(&token);
    }

    // helpers
    fn check_whitelisted(&self, address: &ManagedAddress) {
        self.platform_contract_proxy()
            .contract(self.platform_sc().get())
            .check_whitelisted(address)
            .execute_on_dest_context::<()>();
    }

    // proxies
    #[proxy]
    fn platform_contract_proxy(&self) -> tfn_platform::Proxy<Self::Api>;
}
