use super::shared::transfer_tokens;
use crate::{error::ErrorCode, state::Offer};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct MakeOffer<'info> {
    // MakeOffer (in capitals) is a struct of names accounts that the
    // make_offer() function will use.
    #[account(mut)]
    pub maker: Signer<'info>,

    // `mint::token_program = token_program` allows this account to work with either the classic SPL Token program or the newer Token-2022 extension program.
    #[account(mint::token_program = token_program)]
    pub token_mint_a: InterfaceAccount<'info, Mint>,
    #[account(mint::token_program = token_program)]
    pub token_mint_b: InterfaceAccount<'info, Mint>,

    // token account for maker e.g. USDC for Alice
    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority =  maker,
        associated_token::token_program = token_program
    )]
    pub maker_token_account_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = maker,
        space = Offer::DISCRIMINATOR.len() + Offer::INIT_SPACE,
        seeds = [b"offer", id.to_le_bytes().as_ref()],
        bump
    )]
    pub offer: Account<'info, Offer>,

    #[account(
        init,
        payer = maker,
        associated_token::mint = token_mint_a,
        associated_token::authority =  offer,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    // used to mangage associated token accounts i.e. where a wallet holds a specific type of token
    pub associated_token_program: Program<'info, AssociatedToken>,

    // work with either the classic token program or the newer token extension program
    pub token_program: Interface<'info, TokenInterface>,

    // used to create accounts
    pub system_program: Program<'info, System>,
}

// Handle the make offer instruction by:
// 1. Moving the tokens from the maker's ATA to the vault
// 2. Saving the details of the offer to the offer account
pub fn make_offer(
    context: Context<MakeOffer>,
    id: u64,
    token_a_offered_amount: u64,
    token_b_wanted_amount: u64,
) -> Result<()> {
    // Validate Amount
    require!(token_a_offered_amount > 0, ErrorCode::InvalidAmount);
    require!(token_b_wanted_amount > 0, ErrorCode::InvalidAmount);

    //Validate token mints are different
    require!(
        context.accounts.token_mint_a.key() != context.accounts.token_mint_b.key(),
        ErrorCode::InvalidTokenMint
    );

    // MOve the tokems from the maker's ATA to the vault
    transfer_tokens(
        &context.accounts.maker_token_account_a,
        &context.accounts.vault,
        &token_a_offered_amount,
        &context.accounts.token_mint_a,
        &context.accounts.maker.to_account_info(),
        &context.accounts.token_program,
        None,
    )
    .map_err(|_| ErrorCode::InsufficientMakerBalance)?;

    // Save the details of the offer to the offer account
    context.accounts.offer.set_inner(Offer {
        id,
        maker: context.accounts.maker.key(),
        token_mint_a: context.accounts.token_mint_a.key(),
        token_mint_b: context.accounts.token_mint_b.key(),
        token_b_wanted_amount,
        bump: context.bumps.offer,
    });
    Ok(())
}
