use anchor_lang::prelude::*;

// simple hello world using accounts and instructions.

declare_id!("HWGE5s5rZwNh1B2bKhMDUVVw2qfyv8B2h9EVpLCeKEZt"); // pubkey of program itself

#[program] // entry for instructions
pub mod test_accins {
    use super::*;

    // in-line instruction handler for the hello world
    // we will include a discrim to id this handler
    // we will also include all relevant accounts + program needed via context struct for instruction

    // accounts and program id needed, some optional things to pass in are remaining accts, and the bumsp required for bump seeds for pda addresses 

    #[instruction(discriminator = 2)]
    pub fn hello(ctx: Context<Hello>) -> Result<()> {
        msg!("Program {:?} says hello world!", ctx.program_id);
        Ok(())
    }

    #[instruction(discriminator = 5)]
    pub fn init_vault(ctx: Context<InitVault>) -> Result<()> {

        let vault = &mut ctx.accounts.vault; // changeable vault
        vault.total = 0; // on init its 0
        Ok(())

    }

    // in these instructions, we dont have params of custom type, so no need to worry abt it atm
    #[instruction(discriminator = 3)]
    pub fn deposit(ctx: Context<DepositContext>, val: u64) -> Result<()> {

        // validation for first pass
        require!(val > 0, ErrorType::SubZero);

        // check if payer has enough money check
        let payer_lamports = ctx.accounts.sender.to_account_info().lamports();
        require!(payer_lamports >= val, ErrorType::LackingFunds );

        // dissimilar to the withdraws, since signer has to deposit, the pda seeds not needed here

        let accs = anchor_lang::system_program::Transfer {

            from: ctx.accounts.sender.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),

        };

        // time to do action + sign
        // this is a CPI. We are invoking transfer program!
        anchor_lang::system_program::transfer(

            CpiContext::new(ctx.accounts.system_program.to_account_info(), accs), // program id + acounts for tx
            val,

        )?;

        Ok(())
    }

    #[instruction(discriminator = 4)]
    pub fn withdraw(ctx: Context<WithdrawContext>, val: u64) -> Result<()> {

        require!(val > 0, ErrorType::SubZero);

        let vault_ai = &mut ctx.accounts.vault.to_account_info();
        let receiver_ai = &mut ctx.accounts.receiver.to_account_info();

        let vault_lamports = vault_ai.lamports();
        require!(val <= vault_lamports, ErrorType::TooLarge);

        // safe because your program owns `vault` (Anchor enforces owner)
        **vault_ai.try_borrow_mut_lamports()? -= val;
        **receiver_ai.try_borrow_mut_lamports()? += val;

        Ok(())
    }

    // why do we include discriminator for instruction handlers and context structs? (this is needed, and not wrong btw)
    // why do context structs have lifetimes?
    // 

}


// we will create custom program owned account structs (custom program-owned structs) +  context structs

// custom account structs first

// 1. program owned account struct, have some data here to mutate
#[account]
#[derive(InitSpace)]
pub struct Vault {

    pub total: u8, // this accounts data


}

// context struct with nothing
#[derive(Accounts)]
pub struct Hello {}

// 2. context struct to init the vault
#[derive(Accounts)]
pub struct InitVault<'info> {

    // 1. typed accounts
    #[account(mut)]
    pub sender: Signer<'info>,

    // 2. PDA address account
    // since we are initializing the struct, we need init, cannot be mutable as we are initializing here in the context
    #[account(
        init,
        payer = sender,
        space = Vault::INIT_SPACE + 8, // anchor discriminator size default is 8, init space totals up space used by scalar types at compile
        seeds = [b"vault".as_ref(), sender.key().as_ref()],
        bump, // since we are initializing, and likely to use this pda address account later, we tell anchor to create off curve addy for this
    )]
    pub vault: Account<'info, Vault>,

    // 3. System program
    // system program isnt an account per-say, just a check whether it is the valid prog id
    pub system_program: Program<'info, System>

}

#[derive(Accounts)]
pub struct DepositContext<'info> {

    // so the payer is the signer account here
    #[account(mut)]
    pub sender: Signer<'info>,

    // the receiver is the PDA address account, the vault
    #[account(

        mut,
        seeds = [b"vault".as_ref(), sender.key().as_ref()],
        bump,

    )]
    pub vault: Account<'info, Vault>,

    // ** system_program is the default name, if changed, will lead to err, need to specify the 
    pub system_program: Program<'info, System>

}

#[derive(Accounts)]
pub struct WithdrawContext<'info> {

    pub payer: SystemAccount<'info>, // you may wonder why we need this payer account. Answer: We need it for seed calc!

    #[account(mut)]
    pub receiver: SystemAccount<'info>,

    #[account(

        mut,
        seeds = [b"vault".as_ref(), payer.key.as_ref()],
        bump
 
    )]
    pub vault: Account<'info, Vault>,

    pub system_program: Program<'info, System>,

}

#[error_code]
pub enum ErrorType {

    #[msg("The value is less than or equal to 0")]
    SubZero,
    #[msg("The value is larger than that of account balance")]
    TooLarge,
    #[msg("Insufficient funds")]
    LackingFunds

}