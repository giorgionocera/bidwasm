use cosmwasm_std::{Addr, Coin, StdResult};
use cw_multi_test::{App, ContractWrapper, Executor};

use crate::{
    contract::{execute, instantiate, query},
    msg::{ExecuteMsg, InstantiateMsg},
    ContractError,
};

pub struct BidwasmContract(Addr);

impl BidwasmContract {
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    // Store the code and retrieve the store_code_id
    pub fn store_code(app: &mut App) -> u64 {
        let contract = ContractWrapper::new(execute, instantiate, query);
        app.store_code(Box::new(contract))
    }

    // Perform instantiation for the contract
    #[track_caller]
    pub fn instantiate<'a>(
        app: &mut App,
        code_id: u64,
        sender: &Addr,
        label: &str,
        owner: impl Into<Option<&'a Addr>>,
        denom: &str,
        description: &str,
        commission: impl Into<Option<u128>>,
    ) -> StdResult<Self> {
        let owner = owner.into();
        let commission = commission.into();

        app.instantiate_contract(
            code_id,
            sender.clone(),
            &InstantiateMsg {
                denom: denom.to_string(),
                owner: owner.map(Addr::to_string),
                description: description.to_string(),
                commission,
            },
            &[],
            label,
            None,
        )
        .map(BidwasmContract)
        .map_err(|err| err.downcast().unwrap())
    }

    // Perform bidding to the auction
    #[track_caller]
    pub fn bid(&self, app: &mut App, sender: &Addr, funds: &[Coin]) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecuteMsg::Bid {}, funds)
            .map(|_| ())
            .map_err(|err| err.downcast().unwrap())
    }
}

impl From<BidwasmContract> for Addr {
    fn from(contract: BidwasmContract) -> Self {
        contract.0
    }
}
