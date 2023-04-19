use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;

declare_id!("C8YkV52ZhqwumZUQkp4mbu775nXuq6eos1yB6nJCSD9G");

#[program]
pub mod bet_place {
    use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

    use super::*;

    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        event_id: u32,
        market_id: u32,
        event_name: String,
        market_name: String,
        outcomes: u8,
        line: u16,
        outcome_one: String,
        outcome_two: String,
        outcome_three: String,
        outcome_one_odds: u16,
        outcome_two_odds: u16,
        outcome_three_odds: u16,
    ) -> ProgramResult {

        if outcomes != 2 && outcomes != 3 {
            return Err(ProgramError::Custom(MyError::InvalidOutcomes as u32));
        }

        if outcomes == 2 {
            if outcome_one_odds <= 1000 || outcome_two_odds <= 1000 {
                return Err(ProgramError::Custom(MyError::InvalidOdds as u32));
            }
        
            if (1 / (outcome_one_odds/1000)) + (1 / (outcome_two_odds/1000)) >= 1 {
                return Err(ProgramError::Custom(MyError::InvalidOdds as u32));
            }
        } else {
            if outcome_one_odds <= 1000 || outcome_two_odds <= 1000 || outcome_three_odds <= 1000 {
                return Err(ProgramError::Custom(MyError::InvalidOdds as u32));
            }
        
            if (1 / (outcome_one_odds/1000)) + (1 / (outcome_two_odds/1000)) + (1 / (outcome_three_odds/1000)) >= 1 {
                return Err(ProgramError::Custom(MyError::InvalidOdds as u32));
            }
        }
        

        let market = &mut ctx.accounts.market;
        market.authority = *ctx.accounts.authority.key;
        market.event_id = event_id;
        market.event_name = event_name;
        market.market_id = market_id;
        market.market_name = market_name;
        if outcomes == 2 {
            market.two_way = true;
            market.three_way = false;
            market.outcome_three = None;
            market.outcome_three_odds = None;
        } else {
            market.two_way = false;
            market.three_way = true;
            market.outcome_three = Some(outcome_three);
            market.outcome_three_odds = Some(outcome_three_odds);
        }
        if line == 0 {
            market.line = None;
        } else {
            market.line = Some(line);
        }
        market.outcome_one = outcome_one;
        market.outcome_two = outcome_two;
        
        market.outcome_one_odds = outcome_one_odds;
        market.outcome_two_odds = outcome_two_odds;
        market.open = false;
        market.winning_outcome = 0;
        market.settled = false;
        market.max_win = 1000 * LAMPORTS_PER_SOL;
        market.last_bet_id = 0;

        Ok(())
    }

    pub fn update_odds(
        ctx: Context<UpdateOdds>,
        outcome_one_odds: u16,
        outcome_two_odds: u16,
        outcome_three_odds: u16,
    ) -> ProgramResult {

        if ctx.accounts.market.two_way {
            if outcome_one_odds <= 1000 || outcome_two_odds <= 1000 {
                return Err(ProgramError::Custom(MyError::InvalidOdds as u32));
            }
        } else {
            if outcome_one_odds <= 1000 || outcome_two_odds <= 1000 || outcome_three_odds <= 1000 {
                return Err(ProgramError::Custom(MyError::InvalidOdds as u32));
            }
        }

        let market = &mut ctx.accounts.market;
        if market.authority != *ctx.accounts.authority.key {
            return Err(ProgramError::Custom(MyError::UnauthorizedToUpdateOdds as u32));
        }
        market.outcome_one_odds = outcome_one_odds;
        market.outcome_two_odds = outcome_two_odds;
        if market.three_way {
            market.outcome_three_odds = Some(outcome_three_odds);
        } else {
            market.outcome_three_odds = None;
        }

        Ok(())
    }

    pub fn close_market(ctx: Context<CloseMarket>) -> ProgramResult {
        let market = &mut ctx.accounts.market;
        market.open = false;
        Ok(())
    }

    pub fn open_market(ctx: Context<OpenMarket>) -> ProgramResult {
        let market = &mut ctx.accounts.market;
        market.open = true;
        Ok(())
    }

    pub fn place_bet(ctx: Context<PlaceBet>, event_id: u32, market_id: u32, bet_id: u32, selection: u8, amount: u64) -> ProgramResult {
        if selection != 1 && selection != 2 && selection != 3 {
            return Err(ProgramError::Custom(MyError::InvalidSelection as u32));
        }

        let market = &mut ctx.accounts.market;
        if market.open == false {
            return Err(ProgramError::Custom(MyError::MarketClosed as u32));
        }

        if selection == 3 && market.three_way == false {
            return Err(ProgramError::Custom(MyError::InvalidSelection as u32));
        }

        let odds = match selection {
            1 => market.outcome_one_odds,
            2 => market.outcome_two_odds,
            3 => market.outcome_three_odds.unwrap(),
            _ => 0,
        };

        if u64::from(amount/1000000000) > market.max_win / (u64::from(odds/1000)) {
            return Err(ProgramError::Custom(MyError::StakeTooHigh as u32));
        }

        if bet_id != market.last_bet_id + 1 {
            return Err(ProgramError::Custom(MyError::InvalidBetId as u32));
        }

        market.last_bet_id = bet_id;

        let bet = &mut ctx.accounts.bet;
        bet.authority = market.authority;
        bet.user = *ctx.accounts.authority.key;
        
        bet.event_id = event_id;
        bet.market_id = market_id;
        bet.bet_id = bet_id;
        bet.selection = selection;
        bet.amount = amount;
        bet.odds = odds;
        bet.settled = false;
        bet.result = "Pending".to_string();
        bet.expected_payout = u64::from(amount) * 1_000_000_000 * u64::from(odds) / 1000;

        let txn = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.authority.key(),
            &ctx.accounts.market.key(),
            amount * 1_000_000_000,
        );

        anchor_lang::solana_program::program::invoke(
            &txn,
            &[
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.market.to_account_info(),
            ],
        )?;


        Ok(())
    }

    pub fn settle_market(ctx: Context<SettleMarket>, outcome: u8) -> ProgramResult {
        
        if outcome != 1 && outcome != 2 && outcome != 3 && outcome != 4 {
            return Err(ProgramError::Custom(MyError::InvalidSelection as u32));
        }

        let market = &mut ctx.accounts.market;

        if market.open == true {
            return Err(ProgramError::Custom(MyError::MarketOpen as u32));
        }

        if market.settled == true {
            return Err(ProgramError::Custom(MyError::MarketSettled as u32));
        }

        if market.authority != *ctx.accounts.authority.key {
            return Err(ProgramError::Custom(MyError::UnauthorizedToSettleMarket as u32));
        }

        market.settled = true;

        market.winning_outcome = outcome;

        Ok(())
    }

    pub fn settle_bet(ctx: Context<SettleBet>) -> ProgramResult {

        let market = &mut ctx.accounts.market;

        if market.open == true {
            return Err(ProgramError::Custom(MyError::MarketOpen as u32));
        }

        if market.settled != true {
            return Err(ProgramError::Custom(MyError::MarketNotSettled as u32));
        }

        let bet = &mut ctx.accounts.bet;

        if market.authority != *ctx.accounts.authority.key {
            return Err(ProgramError::Custom(MyError::UnauthorizedToSettleBet as u32));
        }

        if bet.settled == true {
            return Err(ProgramError::Custom(MyError::BetSettled as u32));
        }

        if market.winning_outcome == 4 {
            bet.result = "Void".to_string();
            bet.settled = true;

            let market_account = market.to_account_info();
            let user_account = ctx.accounts.user.to_account_info();

            **market_account.try_borrow_mut_lamports()? += bet.amount * 1_000_000_000;
            **user_account.try_borrow_mut_lamports()? -= bet.amount * 1_000_000_000;
        }
        if bet.selection == market.winning_outcome {

            let market_account = market.to_account_info();
            let user_account = ctx.accounts.user.to_account_info();

            **market_account.try_borrow_mut_lamports()? -= bet.expected_payout;
            **user_account.try_borrow_mut_lamports()? += bet.expected_payout;
            
            bet.result = "Win".to_string();
            bet.settled = true;
        } else {

            bet.result = "Lose".to_string();
            bet.settled = true;
        }

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(event_id: u32, market_id: u32)]
pub struct InitializeMarket<'info> {
    #[account(
        init,
        payer = authority,
        space = 1024,
        seeds = [event_id.to_le_bytes().as_ref(), market_id.to_le_bytes().as_ref(), authority.key().as_ref()],
        bump
    )]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Market {
    pub authority: Pubkey,
    pub event_id: u32,
    pub event_name: String,
    pub market_id: u32,
    pub market_name: String,
    pub two_way: bool,
    pub three_way: bool,
    pub line: Option<u16>,
    pub outcome_one: String,
    pub outcome_two: String,
    pub outcome_three: Option<String>,
    pub outcome_one_odds: u16,
    pub outcome_two_odds: u16,
    pub outcome_three_odds: Option<u16>,
    pub open: bool,
    pub winning_outcome: u8,
    pub settled: bool,
    pub max_win: u64,
    pub last_bet_id: u32,
}

#[derive(Accounts)]
pub struct UpdateOdds<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct CloseMarket<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct OpenMarket<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(event_id:u32, market_id:u32, bet_id: u32)]
pub struct PlaceBet<'info> {
    #[account(
        init,
        payer = authority,
        space = 400,
        seeds = [
            &event_id.to_le_bytes().as_ref(),
            &market_id.to_le_bytes().as_ref(),
            &bet_id.to_le_bytes().as_ref(),
            authority.key().as_ref()
        ],
        bump
    )]
    pub bet: Account<'info, Bet>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Bet {
    pub bet_id: u32,
    pub authority: Pubkey,
    pub user: Pubkey,
    pub market_id: u32,
    pub event_id: u32,
    pub selection: u8,
    pub amount: u64,
    pub settled: bool,
    pub result: String,
    pub odds: u16,
    pub expected_payout: u64,
}

#[derive(Accounts)]
pub struct SettleMarket<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SettleBet<'info> {
    #[account(mut)]
    pub bet: Account<'info, Bet>,
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    /// CHECK: Account info for user is passed from the front end and checked against the bet.user field.
    pub user: AccountInfo<'info>,
}


#[error_code]
pub enum MyError {
    #[msg("Unauthorized to update odds.")]
    UnauthorizedToUpdateOdds = 0,
    #[msg("Invalid Odds.")]
    InvalidOdds = 1,
    #[msg("Invalid Selection.")]
    InvalidSelection = 2,
    #[msg("Market Closed.")]
    MarketClosed = 3,
    #[msg("Stake Too High.")]
    StakeTooHigh = 4,
    #[msg("Bet Already Settled.")]
    BetAlreadySettled = 5,
    #[msg("Market Not Closed.")]
    MarketNotClosed = 6,
    #[msg("Unauthorized to settle bet.")]
    UnauthorizedToSettleBet = 7,
    #[msg("InvalidOutcomes.")]
    InvalidOutcomes = 8,
    #[msg("Market Settled.")]
    MarketSettled = 9,
    #[msg("Bet Settled.")]
    BetSettled = 10,
    #[msg("Market still open.")]
    MarketOpen = 11,
    #[msg("Unauthorised to settle bet.")]
    UnauthorisedToSettleBet = 12,
    #[msg("Invalid Bet ID")]
    InvalidBetId = 13,
    #[msg("Unauthorized to settle market.")]
    UnauthorizedToSettleMarket = 14,
    #[msg("Market Not Settled")]
    MarketNotSettled = 15,
}