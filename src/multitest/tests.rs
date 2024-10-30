use cosmwasm_std::{coins, Addr, Uint128};
use cw_multi_test::App;

use crate::{
    msg::{BidResp, QueryMsg},
    state::{Config, State, Status, BIDS, CONFIG, STATE},
    ContractError,
};

use super::contract::BidwasmContract;

const UATOM: &str = "uatom";

// START --> Auction Opening Tests

#[test]
fn open_auction_with_owner() {
    // Define participants
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::default();

    let code_id = BidwasmContract::store_code(&mut app);

    // Instantiate contract with owner different than sender
    let contract = BidwasmContract::instantiate(
        &mut app,
        code_id,
        &sender,
        "Bidwasm contract",
        &owner,
        UATOM,
        "Supercomputer #2207 bidding",
        500_000,
    )
    .unwrap();

    // Query the contract state
    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();

    // Verify that contract state is correct
    assert_eq!(
        state,
        State {
            current_status: Status::Open,
            highest_bid: None,
        }
    );

    // Query the contract configuration
    let config = CONFIG.query(&app.wrap(), contract.addr().clone()).unwrap();

    // Verify that contract configuration is correct
    assert_eq!(
        config,
        Config {
            denom: UATOM.to_string(),
            owner,
            description: "Supercomputer #2207 bidding".to_string(),
            commission: 500_000
        }
    );
}

#[test]
fn open_auction_without_owner() {
    // Define participant
    let owner = Addr::unchecked("owner");

    let mut app = App::default();

    let code_id = BidwasmContract::store_code(&mut app);

    // Instantiate contract without expressing the owner
    let contract = BidwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidwasm contract",
        None,
        UATOM,
        "Supercomputer #2207 bidding",
        500_000,
    )
    .unwrap();

    // Query the contract state
    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();

    // Verify that contract state is correct
    assert_eq!(
        state,
        State {
            current_status: Status::Open,
            highest_bid: None,
        }
    );

    // Query the contract configuration
    let config = CONFIG.query(&app.wrap(), contract.addr().clone()).unwrap();

    // Verify that contract configuration is correct (if no owner is provided,
    // default owner is the contract creator).
    assert_eq!(
        config,
        Config {
            denom: UATOM.to_string(),
            owner,
            description: "Supercomputer #2207 bidding".to_string(),
            commission: 500_000
        }
    );
}

#[test]
fn open_auction_without_commission() {
    // Define participant
    let owner = Addr::unchecked("owner");

    let mut app = App::default();

    let code_id = BidwasmContract::store_code(&mut app);

    // Instantiate contract without expressing the commission
    let contract = BidwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidwasm contract",
        None,
        UATOM,
        "Supercomputer #2207 bidding",
        None,
    )
    .unwrap();

    // Query the contract state
    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();

    // Verify that contract state is correct
    assert_eq!(
        state,
        State {
            current_status: Status::Open,
            highest_bid: None,
        }
    );

    // Query the contract configuration
    let config = CONFIG.query(&app.wrap(), contract.addr().clone()).unwrap();

    // Verify that contract configuration is correct (if no owner is provided,
    // default owner is the contract creator).
    assert_eq!(
        config,
        Config {
            denom: UATOM.to_string(),
            owner,
            description: "Supercomputer #2207 bidding".to_string(),
            commission: 0
        }
    );
}

// END --> Auction Opening Tests

// START --> Bidding Tests

#[test]
fn owner_cannot_bid() {
    // Define participant
    let owner = Addr::unchecked("owner");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &owner, coins(10_000_000, UATOM))
            .unwrap();
    });

    let code_id = BidwasmContract::store_code(&mut app);

    // Instantiate contract
    let contract = BidwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidwasm contract",
        None,
        UATOM,
        "Supercomputer #2207 bidding",
        500_000,
    )
    .unwrap();

    // Try making the owner bidding to his own auction
    let err = contract
        .bid(&mut app, &owner, &coins(10_000_000, UATOM))
        .unwrap_err();

    // Verify that the contract fails if the contract owner bid to his own
    // auction
    assert_eq!(
        err,
        ContractError::InvalidBid {
            owner: owner.to_string()
        }
    );

    // No funds should be moved
    assert_eq!(
        app.wrap().query_all_balances(owner).unwrap(),
        coins(10_000_000, UATOM)
    );

    assert_eq!(app.wrap().query_all_balances(contract.addr()).unwrap(), &[]);
}

#[test]
fn insufficient_funds_bid() {
    // Define participant
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10_000_000, UATOM))
            .unwrap();
    });

    let code_id = BidwasmContract::store_code(&mut app);

    // Instantiate contract
    let contract = BidwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidwasm contract",
        None,
        UATOM,
        "Supercomputer #2207 bidding",
        500_000,
    )
    .unwrap();

    // Try making the owner bidding to his own auction
    let err = contract
        .bid(&mut app, &sender, &coins(100_000, UATOM))
        .unwrap_err();

    // Verify that the contract fails if the sender sends lower funds than
    // required for the commission by the auction
    assert_eq!(
        err,
        ContractError::InsufficientFundsForCommission {
            funds: Uint128::new(100_000),
            commission: Uint128::new(500_000)
        }
    );

    // No funds should be moved
    assert_eq!(
        app.wrap().query_all_balances(sender).unwrap(),
        coins(10_000_000, UATOM)
    );

    assert_eq!(app.wrap().query_all_balances(owner).unwrap(), &[]);

    assert_eq!(app.wrap().query_all_balances(contract.addr()).unwrap(), &[]);
}

#[test]
fn simple_bid_no_commission() {
    // Define participant
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(3_500_000, UATOM))
            .unwrap();
    });

    let code_id = BidwasmContract::store_code(&mut app);

    // Instantiate contract
    let contract = BidwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidwasm contract",
        None,
        UATOM,
        "Supercomputer #2207 bidding",
        0,
    )
    .unwrap();

    // Making a simple bid
    contract
        .bid(&mut app, &sender, &coins(3_500_000, UATOM))
        .unwrap();

    let bid = BIDS
        .query(&app.wrap(), contract.addr().clone(), &sender)
        .unwrap();

    // Check if bid is stored in the state
    assert_eq!(bid, Some(Uint128::new(3_500_000)));

    // sender should have not any balance
    assert_eq!(app.wrap().query_all_balances(sender).unwrap(), &[]);

    // owner should have not any balance
    assert_eq!(app.wrap().query_all_balances(owner).unwrap(), &[]);

    // contract should store the whole bid (no commission)
    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        coins(3_500_000, UATOM)
    );
}

#[test]
fn simple_bid() {
    // Define participant
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(3_500_000, UATOM))
            .unwrap();
    });

    let code_id = BidwasmContract::store_code(&mut app);

    // Instantiate contract
    let contract = BidwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidwasm contract",
        None,
        UATOM,
        "Supercomputer #2207 bidding",
        500_000,
    )
    .unwrap();

    // Making a simple bid
    contract
        .bid(&mut app, &sender, &coins(3_500_000, UATOM))
        .unwrap();

    let bid = BIDS
        .query(&app.wrap(), contract.addr().clone(), &sender)
        .unwrap();

    // Check if bid is stored in the state
    assert_eq!(bid, Some(Uint128::new(3_000_000)));

    // sender should have not any balance
    assert_eq!(app.wrap().query_all_balances(sender).unwrap(), &[]);

    // owner should have got the commission
    assert_eq!(
        app.wrap().query_all_balances(owner).unwrap(),
        coins(500_000, UATOM)
    );

    // contract should store bid minus commission
    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        coins(3_000_000, UATOM)
    );
}

// END --> Bidding Tests

// START --> Close Tests

#[test]
fn close_auction() {
    // Define participant
    let owner = Addr::unchecked("owner");

    let mut app = App::default();

    let code_id = BidwasmContract::store_code(&mut app);

    // Instantiate contract
    let contract = BidwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidwasm contract",
        None,
        UATOM,
        "Supercomputer #2207 bidding",
        500_000,
    )
    .unwrap();

    // Check the status is open
    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();
    assert_eq!(
        state,
        State {
            current_status: Status::Open,
            highest_bid: None
        }
    );

    // Close the auction
    contract.close(&mut app, &owner).unwrap();

    // Check the status is closed
    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();
    assert_eq!(
        state,
        State {
            current_status: Status::Closed,
            highest_bid: None
        }
    );
}

#[test]
fn close_auction_after_bid() {
    // Define participant
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(4_000_000, UATOM))
            .unwrap();
    });

    let code_id = BidwasmContract::store_code(&mut app);

    // Instantiate contract
    let contract = BidwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidwasm contract",
        None,
        UATOM,
        "Supercomputer #2207 bidding",
        500_000,
    )
    .unwrap();

    // Making a simple bid
    contract
        .bid(&mut app, &sender, &coins(2_000_000, UATOM))
        .unwrap();

    let bid = BIDS
        .query(&app.wrap(), contract.addr().clone(), &sender)
        .unwrap();

    // Check if bid is stored in the state
    assert_eq!(bid, Some(Uint128::new(1_500_000)));

    // sender should have not any balance
    assert_eq!(
        app.wrap().query_all_balances(&sender).unwrap(),
        coins(2_000_000, UATOM)
    );

    // owner should have got the commission
    assert_eq!(
        app.wrap().query_all_balances(&owner).unwrap(),
        coins(500_000, UATOM)
    );

    // contract should store bid minus commission
    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        coins(1_500_000, UATOM)
    );

    // Check the status is open
    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();
    assert_eq!(
        state,
        State {
            current_status: Status::Open,
            highest_bid: Some((sender.clone(), Uint128::new(1_500_000)))
        }
    );

    // Close the auction
    contract.close(&mut app, &owner).unwrap();

    // Check the status is closed
    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();
    assert_eq!(
        state,
        State {
            current_status: Status::Closed,
            highest_bid: Some((sender.clone(), Uint128::new(1_500_000)))
        }
    );

    // sender should have the balance minus bid and commission
    assert_eq!(
        app.wrap().query_all_balances(&sender).unwrap(),
        coins(2_000_000, UATOM)
    );

    // owner should have got the commission and the bid
    assert_eq!(
        app.wrap().query_all_balances(&owner).unwrap(),
        coins(2_000_000, UATOM)
    );

    // contract should store nothing
    assert_eq!(app.wrap().query_all_balances(contract.addr()).unwrap(), &[]);

    // Try making another bid after closing the auction
    let err = contract
        .bid(&mut app, &sender, &coins(2_000_000, UATOM))
        .unwrap_err();

    // Verify that the contract fails if the sender bids after the acution is
    // closed
    assert_eq!(err, ContractError::ClosedAcution);
}

#[test]
fn invalid_close_unauthorized() {
    // Define participant
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::default();

    let code_id = BidwasmContract::store_code(&mut app);

    // Instantiate contract
    let contract = BidwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidwasm contract",
        None,
        UATOM,
        "Supercomputer #2207 bidding",
        500_000,
    )
    .unwrap();

    // Check the status is open
    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();
    assert_eq!(
        state,
        State {
            current_status: Status::Open,
            highest_bid: None
        }
    );

    // Try closing the auction from unauthorized account
    let err = contract.close(&mut app, &sender).unwrap_err();

    // Verify that the contract fails if the auction is closed by unauthorized
    // address
    assert_eq!(
        err,
        ContractError::Unauthorized {
            owner: owner.to_string()
        }
    );

    // Check the status is open
    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();
    assert_eq!(
        state,
        State {
            current_status: Status::Open,
            highest_bid: None
        }
    );
}

// END --> Close Tests

// START --> Retract Tests

#[test]
fn retract() {
    // Define participant
    let owner = Addr::unchecked("owner");
    let sender1 = Addr::unchecked("sender1");
    let sender2 = Addr::unchecked("sender2");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender1, coins(4_500_000, UATOM))
            .unwrap();
        router
            .bank
            .init_balance(storage, &sender2, coins(7_500_000, UATOM))
            .unwrap();
    });

    let code_id = BidwasmContract::store_code(&mut app);

    // Instantiate contract
    let contract = BidwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidwasm contract",
        None,
        UATOM,
        "Supercomputer #2207 bidding",
        500_000,
    )
    .unwrap();

    // Sender1 make a bid of 4_000_000
    contract
        .bid(&mut app, &sender1, &coins(4_500_000, UATOM))
        .unwrap();

    // Sender2 make a bid of 7_000_000
    contract
        .bid(&mut app, &sender2, &coins(7_500_000, UATOM))
        .unwrap();

    // Check that the bids are registered correctly
    let sender1_bid = BIDS
        .query(&app.wrap(), contract.addr().clone(), &sender1)
        .unwrap();

    assert_eq!(sender1_bid, Some(Uint128::new(4_000_000)));

    let sender2_bid = BIDS
        .query(&app.wrap(), contract.addr().clone(), &sender2)
        .unwrap();

    assert_eq!(sender2_bid, Some(Uint128::new(7_000_000)));

    // senders should have not any balance
    assert_eq!(app.wrap().query_all_balances(&sender1).unwrap(), &[]);
    assert_eq!(app.wrap().query_all_balances(&sender2).unwrap(), &[]);

    // owner should have got the commission
    assert_eq!(
        app.wrap().query_all_balances(&owner).unwrap(),
        coins(1_000_000, UATOM)
    );

    // contract should store bid minus commission
    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        coins(11_000_000, UATOM)
    );

    // Close the auction
    contract.close(&mut app, &owner).unwrap();

    // Check the status is closed
    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();
    assert_eq!(
        state,
        State {
            current_status: Status::Closed,
            highest_bid: Some((sender2.clone(), Uint128::new(7_000_000)))
        }
    );

    // Sender1 retracting funds since sender did not win the auction
    contract.retract(&mut app, &sender1, None).unwrap();

    // Sender1 should have the original balance minus the commission for the
    // bid
    assert_eq!(
        app.wrap().query_all_balances(&sender1).unwrap(),
        coins(4_000_000, UATOM)
    );

    // Sender2 should have not any balance
    assert_eq!(app.wrap().query_all_balances(&sender2).unwrap(), &[]);

    // owner should have got the commission plus the highest bid at the time
    // of the auction closure
    assert_eq!(
        app.wrap().query_all_balances(&owner).unwrap(),
        coins(8_000_000, UATOM)
    );

    // contract should have not any balance
    assert_eq!(app.wrap().query_all_balances(contract.addr()).unwrap(), &[]);
}

#[test]
fn retract_on_another_recipient() {
    // Define participant
    let owner = Addr::unchecked("owner");
    let sender1 = Addr::unchecked("sender1");
    let sender2 = Addr::unchecked("sender2");
    let recipient = Addr::unchecked("recipient");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender1, coins(4_500_000, UATOM))
            .unwrap();
        router
            .bank
            .init_balance(storage, &sender2, coins(7_500_000, UATOM))
            .unwrap();
    });

    let code_id = BidwasmContract::store_code(&mut app);

    // Instantiate contract
    let contract = BidwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidwasm contract",
        None,
        UATOM,
        "Supercomputer #2207 bidding",
        500_000,
    )
    .unwrap();

    // Sender1 make a bid of 4_000_000
    contract
        .bid(&mut app, &sender1, &coins(4_500_000, UATOM))
        .unwrap();

    // Sender2 make a bid of 7_000_000
    contract
        .bid(&mut app, &sender2, &coins(7_500_000, UATOM))
        .unwrap();

    // Close the auction
    contract.close(&mut app, &owner).unwrap();

    // Sender1 retracting funds since sender did not win the auction
    contract.retract(&mut app, &sender1, &recipient).unwrap();

    // Recipient should have the original balance minus the commission for the
    // bid
    assert_eq!(
        app.wrap().query_all_balances(&recipient).unwrap(),
        coins(4_000_000, UATOM)
    );

    // Sender1 and Sender2 should have not any balance
    assert_eq!(app.wrap().query_all_balances(&sender1).unwrap(), &[]);
    assert_eq!(app.wrap().query_all_balances(&sender2).unwrap(), &[]);

    // owner should have got the commission plus the highest bid at the time
    // of the auction closure
    assert_eq!(
        app.wrap().query_all_balances(&owner).unwrap(),
        coins(8_000_000, UATOM)
    );

    // contract should have not any balance
    assert_eq!(app.wrap().query_all_balances(contract.addr()).unwrap(), &[]);
}

#[test]
fn invalid_retract_by_winner() {
    // Define participant
    let owner = Addr::unchecked("owner");
    let sender1 = Addr::unchecked("sender1");
    let sender2 = Addr::unchecked("sender2");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender1, coins(4_500_000, UATOM))
            .unwrap();
        router
            .bank
            .init_balance(storage, &sender2, coins(7_500_000, UATOM))
            .unwrap();
    });

    let code_id = BidwasmContract::store_code(&mut app);

    // Instantiate contract
    let contract = BidwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidwasm contract",
        None,
        UATOM,
        "Supercomputer #2207 bidding",
        500_000,
    )
    .unwrap();

    // Sender1 make a bid of 4_000_000
    contract
        .bid(&mut app, &sender1, &coins(4_500_000, UATOM))
        .unwrap();

    // Sender2 make a bid of 7_000_000
    contract
        .bid(&mut app, &sender2, &coins(7_500_000, UATOM))
        .unwrap();

    // Close the auction
    contract.close(&mut app, &owner).unwrap();

    // Sender2 tries retracting funds even if he won the auction
    let err = contract.retract(&mut app, &sender2, None).unwrap_err();

    // Verify that the contract fails
    assert_eq!(err, ContractError::InvalidRetract);
}

#[test]
fn invalid_double_retract() {
    // Define participant
    let owner = Addr::unchecked("owner");
    let sender1 = Addr::unchecked("sender1");
    let sender2 = Addr::unchecked("sender2");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender1, coins(4_500_000, UATOM))
            .unwrap();
        router
            .bank
            .init_balance(storage, &sender2, coins(7_500_000, UATOM))
            .unwrap();
    });

    let code_id = BidwasmContract::store_code(&mut app);

    // Instantiate contract
    let contract = BidwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidwasm contract",
        None,
        UATOM,
        "Supercomputer #2207 bidding",
        500_000,
    )
    .unwrap();

    // Sender1 make a bid of 4_000_000
    contract
        .bid(&mut app, &sender1, &coins(4_500_000, UATOM))
        .unwrap();

    // Sender2 make a bid of 7_000_000
    contract
        .bid(&mut app, &sender2, &coins(7_500_000, UATOM))
        .unwrap();

    // Close the auction
    contract.close(&mut app, &owner).unwrap();

    // Sender1 retract funds since he did not win the auction
    contract.retract(&mut app, &sender1, None).unwrap();

    // Sender1 tries retracting again the funds even if he already retract them
    let err = contract.retract(&mut app, &sender2, None).unwrap_err();

    // Verify that the contract fails
    assert_eq!(err, ContractError::InvalidRetract);
}

#[test]
fn invalid_retract_while_open() {
    // Define participant
    let owner = Addr::unchecked("owner");
    let sender1 = Addr::unchecked("sender1");
    let sender2 = Addr::unchecked("sender2");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender1, coins(4_500_000, UATOM))
            .unwrap();
        router
            .bank
            .init_balance(storage, &sender2, coins(7_500_000, UATOM))
            .unwrap();
    });

    let code_id = BidwasmContract::store_code(&mut app);

    // Instantiate contract
    let contract = BidwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidwasm contract",
        None,
        UATOM,
        "Supercomputer #2207 bidding",
        500_000,
    )
    .unwrap();

    // Sender1 make a bid of 4_000_000
    contract
        .bid(&mut app, &sender1, &coins(4_500_000, UATOM))
        .unwrap();

    // Sender2 make a bid of 7_000_000
    contract
        .bid(&mut app, &sender2, &coins(7_500_000, UATOM))
        .unwrap();

    // Sender2 tries retracting funds even if he won the auction
    let err = contract.retract(&mut app, &sender2, None).unwrap_err();

    // Verify that the contract fails
    assert_eq!(err, ContractError::OpenAcution);
}
// END --> Retract Tests

// START --> Query Tests
#[test]
fn query_total_bid() {
    // Define participant
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(7_000_000, UATOM))
            .unwrap();
    });

    let code_id = BidwasmContract::store_code(&mut app);

    // Instantiate contract
    let contract = BidwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidwasm contract",
        None,
        UATOM,
        "Supercomputer #2207 bidding",
        500_000,
    )
    .unwrap();

    let resp = contract.query_total_bid(&app, &sender).unwrap();
    assert_eq!(resp, Uint128::new(0));

    // Making a simple bid
    contract
        .bid(&mut app, &sender, &coins(3_500_000, UATOM))
        .unwrap();

    let resp: Uint128 = app
        .wrap()
        .query_wasm_smart(
            contract.addr(),
            &QueryMsg::TotalBid {
                address: sender.to_string(),
            },
        )
        .unwrap();

    assert_eq!(resp, Uint128::new(3_000_000));

    // Making another bid
    contract
        .bid(&mut app, &sender, &coins(3_500_000, UATOM))
        .unwrap();

    // Verify total bid for sender
    let resp = contract.query_total_bid(&app, &sender).unwrap();
    assert_eq!(resp, Uint128::new(6_000_000));
}

#[test]
fn query_highest_bid() {
    // Define participant
    let owner = Addr::unchecked("owner");
    let sender1 = Addr::unchecked("sender1");
    let sender2 = Addr::unchecked("sender2");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender1, coins(4_500_000, UATOM))
            .unwrap();
        router
            .bank
            .init_balance(storage, &sender2, coins(7_500_000, UATOM))
            .unwrap();
    });

    let code_id = BidwasmContract::store_code(&mut app);

    // Instantiate contract
    let contract = BidwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidwasm contract",
        None,
        UATOM,
        "Supercomputer #2207 bidding",
        500_000,
    )
    .unwrap();

    // Verify the highest_bid query returns an error if there is not any bid
    let err = contract.query_highest_bid(&app).unwrap_err();
    assert!(err.to_string().contains("The auction has not any bid"));

    // Sender1 make a bid of 4_000_000
    contract
        .bid(&mut app, &sender1, &coins(4_500_000, UATOM))
        .unwrap();

    // Sender2 make a bid of 7_000_000
    contract
        .bid(&mut app, &sender2, &coins(7_500_000, UATOM))
        .unwrap();

    // Verify highest bid query
    let resp = contract.query_highest_bid(&app).unwrap();
    assert_eq!(
        resp,
        BidResp {
            address: sender2,
            amount: Uint128::new(7_000_000)
        }
    );
}

#[test]
fn query_status() {
    // Define participant
    let owner = Addr::unchecked("owner");

    let mut app = App::default();

    let code_id = BidwasmContract::store_code(&mut app);

    // Instantiate contract
    let contract = BidwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidwasm contract",
        None,
        UATOM,
        "Supercomputer #2207 bidding",
        500_000,
    )
    .unwrap();

    // Verify the is_closed query to check that the auction is not closed
    let resp = contract.query_is_closed(&app).unwrap();
    assert_eq!(resp, false);

    // Close the auction
    contract.close(&mut app, &owner).unwrap();

    // Verify the is_closed query to check that the auction is now closed
    let resp = contract.query_is_closed(&app).unwrap();
    assert_eq!(resp, true);
}

#[test]
fn query_winner() {
    // Define participants
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(4_500_000, UATOM))
            .unwrap();
    });

    let code_id = BidwasmContract::store_code(&mut app);

    // Instantiate contract
    let contract = BidwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidwasm contract",
        None,
        UATOM,
        "Supercomputer #2207 bidding",
        500_000,
    )
    .unwrap();

    // Sender1 make a bid of 4_000_000
    contract
        .bid(&mut app, &sender, &coins(4_500_000, UATOM))
        .unwrap();

    // Verify the winner query returns an error if the auction is yet open
    let err = contract.query_winner(&app).unwrap_err();
    assert!(err.to_string().contains("The auction is yet open"));

    // Close the auction
    contract.close(&mut app, &owner).unwrap();

    // Verify the is_closed query to check that the auction is now closed
    let resp = contract.query_winner(&app).unwrap();
    assert_eq!(
        resp,
        BidResp {
            address: sender,
            amount: Uint128::new(4_000_000)
        }
    );
}

#[test]
fn query_no_winner() {
    // Define participants
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(4_500_000, UATOM))
            .unwrap();
    });

    let code_id = BidwasmContract::store_code(&mut app);

    // Instantiate contract
    let contract = BidwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidwasm contract",
        None,
        UATOM,
        "Supercomputer #2207 bidding",
        500_000,
    )
    .unwrap();

    // Close the auction
    contract.close(&mut app, &owner).unwrap();

    // Verify the winner query returns an error if the auction has not any bid
    let err = contract.query_winner(&app).unwrap_err();
    assert!(err.to_string().contains("The auction has not any bid"));
}
// END --> Query Tests

// START --> Complete Test
#[test]
fn complete_test() {
    // Define participants
    let owner = Addr::unchecked("owner");
    let alex = Addr::unchecked("alex");
    let ann = Addr::unchecked("ann");
    let ann_friend = Addr::unchecked("ann_friend");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &alex, coins(20_000_000, UATOM))
            .unwrap();
        router
            .bank
            .init_balance(storage, &ann, coins(20_000_000, UATOM))
            .unwrap();
    });

    let code_id = BidwasmContract::store_code(&mut app);

    // Instantiate contract
    let contract = BidwasmContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Bidwasm contract",
        None,
        UATOM,
        "Supercomputer #2207 bidding",
        None,
    )
    .unwrap();

    // Alex is sending bid {} message with 15 atoms
    contract
        .bid(&mut app, &alex, &coins(15_000_000, UATOM))
        .unwrap();

    // The highest bid right now is 15 atoms by Alex
    let resp = contract.query_highest_bid(&app).unwrap();
    assert_eq!(
        resp,
        BidResp {
            address: alex.clone(),
            amount: Uint128::new(15_000_000)
        }
    );

    // Now Ann is sending bid {} message with 17 atoms
    contract
        .bid(&mut app, &ann, &coins(17_000_000, UATOM))
        .unwrap();

    // The highest bid is 17 atoms by Ann
    let resp = contract.query_highest_bid(&app).unwrap();
    assert_eq!(
        resp,
        BidResp {
            address: ann.clone(),
            amount: Uint128::new(17_000_000)
        }
    );

    // and total bid by Alex is 15 atoms
    let resp = contract.query_total_bid(&app, &alex).unwrap();
    assert_eq!(resp, Uint128::new(15_000_000));

    // Now Ann is sending another bid {} message with 2 atoms
    contract
        .bid(&mut app, &ann, &coins(2_000_000, UATOM))
        .unwrap();

    // Now the highest bid is 19 atoms by Ann
    let resp = contract.query_highest_bid(&app).unwrap();
    assert_eq!(
        resp,
        BidResp {
            address: ann.clone(),
            amount: Uint128::new(19_000_000)
        }
    );

    // and total of Alex is 15 atoms
    let resp = contract.query_total_bid(&app, &alex).unwrap();
    assert_eq!(resp, Uint128::new(15_000_000));

    // Then Alex sends bid {} message with 1 atom - this message fails, as it
    // would leave Alex at 16 atoms bid total, which is not the highest right
    // now. He has to send more than 5 atoms
    contract
        .bid(&mut app, &alex, &coins(1_000_000, UATOM))
        .unwrap_err();

    // Alex sends another bid {} with 5 atoms
    contract
        .bid(&mut app, &alex, &coins(5_000_000, UATOM))
        .unwrap();

    // It makes the highest bid being 20 atoms by Alex
    let resp = contract.query_highest_bid(&app).unwrap();
    assert_eq!(
        resp,
        BidResp {
            address: alex.clone(),
            amount: Uint128::new(20_000_000)
        }
    );

    // and Ann has total of 19 atoms bid
    let resp = contract.query_total_bid(&app, &ann).unwrap();
    assert_eq!(resp, Uint128::new(19_000_000));

    // The close {} is send by contract owner
    contract.close(&mut app, &owner).unwrap();
    assert_eq!(
        app.wrap().query_all_balances(&owner).unwrap(),
        coins(20_000_000, UATOM)
    );

    // Ann can claim her atoms back calling retract {} message, optionally
    // putting a receiver field there to point where funds should be send back
    // to.
    contract.retract(&mut app, &ann, &ann_friend).unwrap();
    assert_eq!(
        app.wrap().query_all_balances(&ann_friend).unwrap(),
        coins(19_000_000, UATOM)
    );
}
// END --> Complete Test
