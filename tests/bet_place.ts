import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { BetPlace } from "../target/types/bet_place";
import { assert, expect } from "chai";
import { LAMPORTS_PER_SOL } from "@solana/web3.js";

const event_id = Math.floor(Math.random() * 10000);
const market_id = Math.floor(Math.random() * 10000);

describe("bet_place", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.BetPlace as Program<BetPlace>;

  it("Is initialized!", async () => {
    const event_name = "The Blues vs The Reds";
    const market_name = "Match Winner";
    const outcomes = 3;
    const line = 0;
    const outcome_one = "Home";
    const outcome_two = "Draw";
    const outcome_three = "Away";
    const outcome_one_odds = 2960;
    const outcome_two_odds = 3750;
    const outcome_three_odds = 2520;
    const authority = anchor.AnchorProvider.local().wallet.publicKey;
    const [market_pda] = await anchor.web3.PublicKey.findProgramAddressSync([
      new anchor.BN(event_id).toBuffer('le', 4),
      new anchor.BN(market_id).toBuffer('le', 4),
      authority.toBuffer(),
    ],
      program.programId
    );

    const tx = await program.methods.initializeMarket(
      event_id,
      market_id,
      event_name,
      market_name,
      outcomes,
      line,
      outcome_one,
      outcome_two,
      outcome_three,
      outcome_one_odds,
      outcome_two_odds,
      outcome_three_odds).accounts({
        authority: authority,
        market: market_pda,
        systemProgram: anchor.web3.SystemProgram.programId,
      }).rpc();

    const market = await program.account.market.fetch(market_pda);
    console.log(market);

    assert.equal(market.authority.toBase58(), authority.toBase58());
    assert.equal(market.eventId, event_id);
    assert.equal(market.marketId, market_id);
    assert.equal(market.eventName, event_name);
    assert.equal(market.marketName, market_name);
    assert.equal(market.twoWay, false);
    assert.equal(market.threeWay, true);
    assert.isNull(market.line)
    assert.equal(market.outcomeOne, outcome_one);
    assert.equal(market.outcomeTwo, outcome_two);
    assert.equal(market.outcomeThree, outcome_three);
    assert.equal(market.outcomeOneOdds, outcome_one_odds);
    assert.equal(market.outcomeTwoOdds, outcome_two_odds);
    assert.equal(market.outcomeThreeOdds, outcome_three_odds);
    assert.equal(market.open, false);
    assert.equal(market.result, "Pending");
    assert.equal(market.settled, false);
    console.log("Max Win: ", market.maxWin.toNumber() / LAMPORTS_PER_SOL, " SOL");
    console.log("Last Bet ID: ", market.lastBetId);
  });

  it("Can open the market!", async () => {


    const authority = anchor.AnchorProvider.local().wallet.publicKey;
    const [market_pda] = await anchor.web3.PublicKey.findProgramAddressSync([
      new anchor.BN(event_id).toBuffer('le', 4),
      new anchor.BN(market_id).toBuffer('le', 4),
      authority.toBuffer(),
    ],
      program.programId
    );

    const tx = await program.methods.openMarket().accounts({
      authority: authority,
      market: market_pda,
    }).rpc();

    const market = await program.account.market.fetch(market_pda);
    assert.equal(market.open, true);
    console.log("Market.open: ", market.open);
  });

  it("Can close the market!", async () => {

    const authority = anchor.AnchorProvider.local().wallet.publicKey;
    const [market_pda] = await anchor.web3.PublicKey.findProgramAddressSync([
      new anchor.BN(event_id).toBuffer('le', 4),
      new anchor.BN(market_id).toBuffer('le', 4),
      authority.toBuffer(),
    ],
      program.programId
    );

    const tx = await program.methods.closeMarket().accounts({
      authority: authority,
      market: market_pda,
    }).rpc();

    const market = await program.account.market.fetch(market_pda);
    assert.equal(market.open, false);
    console.log("Market,open: ", market.open);
  });

  it("Can re-open the market for bets!", async () => {


    const authority = anchor.AnchorProvider.local().wallet.publicKey;
    const [market_pda] = await anchor.web3.PublicKey.findProgramAddressSync([
      new anchor.BN(event_id).toBuffer('le', 4),
      new anchor.BN(market_id).toBuffer('le', 4),
      authority.toBuffer(),
    ],
      program.programId
    );

    const tx = await program.methods.openMarket().accounts({
      authority: authority,
      market: market_pda,
    }).rpc();

    const market = await program.account.market.fetch(market_pda);
    assert.equal(market.open, true);
    console.log("Market.open: ", market.open);
  });

  it("Can update odds!", async () => {

    const outcome_one_odds = 2960;
    const outcome_two_odds = 2520;
    const outcome_three_odds = 3750;
    const authority = anchor.AnchorProvider.local().wallet.publicKey;
    const [market_pda] = await anchor.web3.PublicKey.findProgramAddressSync([
      new anchor.BN(event_id).toBuffer('le', 4),
      new anchor.BN(market_id).toBuffer('le', 4),
      authority.toBuffer(),
    ],
      program.programId
    );

    const tx = await program.methods.updateOdds(
      outcome_one_odds,
      outcome_two_odds,
      outcome_three_odds,
    ).accounts({
      authority: authority,
      market: market_pda,
    }).rpc();

    const market = await program.account.market.fetch(market_pda);
    console.log(market);
    assert.equal(market.authority.toBase58(), authority.toBase58());


    assert.equal(market.outcomeOneOdds, outcome_one_odds);
    assert.equal(market.outcomeTwoOdds, outcome_two_odds);
    assert.equal(market.outcomeThreeOdds, outcome_three_odds);


    const initialPDABalance = await program.provider.connection.getBalance(market_pda);
    const airdrop = await program.provider.connection.requestAirdrop(market_pda, 100 * anchor.web3.LAMPORTS_PER_SOL);
    const latestBlockHash = await program.provider.connection.getLatestBlockhash();
    await program.provider.connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: airdrop
    });

    const newPDABalance = await program.provider.connection.getBalance(market_pda);
    console.log("New PDA balance: ", newPDABalance / LAMPORTS_PER_SOL, " SOL");

  });

  it("Can place a bet!", async () => {

    const authority = anchor.AnchorProvider.local().wallet.publicKey;
    const [market_pda] = await anchor.web3.PublicKey.findProgramAddressSync([
      new anchor.BN(event_id).toBuffer('le', 4),
      new anchor.BN(market_id).toBuffer('le', 4),
      authority.toBuffer(),
    ],
      program.programId
    );

    const airdrop = await program.provider.connection.requestAirdrop(authority, 100 * anchor.web3.LAMPORTS_PER_SOL);
    const latestBlockHash = await program.provider.connection.getLatestBlockhash();
    await program.provider.connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: airdrop
    });

    console.log("Bettor Balance: ", (await program.provider.connection.getBalance(authority)) / LAMPORTS_PER_SOL, " SOL")


    const market = await program.account.market.fetch(market_pda);
    const bet_id = market.lastBetId + 1;

    const [bet_pda] = await anchor.web3.PublicKey.findProgramAddressSync([
      new anchor.BN(event_id).toBuffer('le', 4),
      new anchor.BN(market_id).toBuffer('le', 4),
      new anchor.BN(bet_id).toBuffer('le', 4),
      authority.toBuffer(),
    ],
      program.programId
    );

    const initialMarketBalance = await program.provider.connection.getBalance(market_pda);
    const initialBettorBalance = await program.provider.connection.getBalance(authority);
    console.log("Initial market_pda balance: ", initialMarketBalance / LAMPORTS_PER_SOL, " SOL");

    const amount = new BN(10);
    const tx = await program.methods.placeBet(
      event_id,
      market_id,
      bet_id,
      1,
      amount
    ).accounts({
      authority: authority,
      market: market_pda,
      bet: bet_pda,
    }).rpc();

    const newMarketBalance = await program.provider.connection.getBalance(market_pda);
    console.log("New book balance: ", newMarketBalance / LAMPORTS_PER_SOL, " SOL");
    assert.equal(newMarketBalance, initialMarketBalance + amount.toNumber());
    console.log("Book balance is correct: ", newMarketBalance / LAMPORTS_PER_SOL, " SOL");
    // const newBettorBalance = await program.provider.connection.getBalance(authority);
    // assert.equal(newBettorBalance, initialBettorBalance - amount);
    // console.log("Bettor balance is correct: ", newBettorBalance / LAMPORTS_PER_SOL, " SOL");

    const bet = await program.account.bet.fetch(bet_pda);
    assert.equal(bet.betId, bet_id);
    console.log("Bet ID: ", bet.betId);
    assert.equal(bet.authority.toBase58(), authority.toBase58());
    console.log("Bet Authority: ", bet.authority.toBase58());
    assert.equal(bet.marketId, market_id);
    console.log("Bet Market ID: ", bet.marketId);
    assert.equal(bet.eventId, event_id);
    console.log("Bet Event ID: ", bet.eventId);
    assert.equal(bet.selection, 1);
    console.log("Bet Selection: ", bet.selection);
    assert.equal(bet.amount.toNumber(), amount.toNumber());
    console.log("Bet Amount: ", bet.amount.toNumber(), " SOL");
    assert.equal(bet.settled, false);
    console.log("Bet Settled: ", bet.settled);
    assert.equal(bet.result, "Pending");
    console.log("Bet Result: ", bet.result);
    assert.equal(bet.odds, market.outcomeOneOdds);
    console.log("Bet Odds: ", bet.odds / 1000);
    const payout = new BN(amount.toNumber() * LAMPORTS_PER_SOL *  market.outcomeOneOdds / 1000)
    assert.equal(bet.expectedPayout.toNumber(), payout.toNumber());
    console.log("Bet Expected Payout: ", bet.expectedPayout.toNumber() / LAMPORTS_PER_SOL, " SOL");

  });

  // it("Can settle a bet!", async () => {
  //   const market = "1X2";
  //   const authority = anchor.AnchorProvider.local().wallet.publicKey;
  //   const [book_pda] = await anchor.web3.PublicKey.findProgramAddressSync([
  //     anchor.utils.bytes.utf8.encode(market),
  //     authority.toBuffer(),
  //   ],
  //   program.programId
  //   );

  //   const [bet_pda] = await anchor.web3.PublicKey.findProgramAddressSync([
  //     anchor.utils.bytes.utf8.encode("Home"),
  //     authority.toBuffer(),
  //   ],
  //   program.programId
  //   );

  //   const amount = new BN(10000);
  //   const placeBetTx = await program.methods.placeBet("Home", amount).accounts({
  //     authority: authority,
  //     book: book_pda,
  //     bet: bet_pda,
  //   }).rpc();

  //   const initialBookBalance = await program.provider.connection.getBalance(book_pda);
  //   console.log("Initial book balance: ", initialBookBalance)

  //   const bet = await program.account.bet.fetch(bet_pda);

  //   console.log(bet.user.toBase58());
  //   console.log(authority.toBase58());

  //   const payout = bet.win;
  //   const tx = await program.methods.settleBet("Home").accounts({
  //     authority: authority,
  //     user: bet.user,
  //     book: book_pda,
  //     bet: bet_pda,
  //   }).rpc({ commitment: "confirmed" });

  //   const newBookBalance = await program.provider.connection.getBalance(book_pda);
  //   console.log("New bet balance: ", newBookBalance)
  //   assert.equal(newBookBalance, initialBookBalance - payout.toNumber());
  //   console.log("Book balance is correct: ", newBookBalance);
  //   console.log("Bet payout is correct: ", bet.win.toNumber());


  //   assert.equal(bet.state, "Settled");
  //   console.log("Bet state is correct: ", bet.state);

  // });


});

