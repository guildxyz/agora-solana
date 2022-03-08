import { PublicKey } from "@solana/web3.js";
import fetch from "node-fetch";
// @ts-ignore
global.fetch = fetch;
// @ts-ignore
global.Headers = fetch.Headers;
// @ts-ignore
global.Request = fetch.Request;
// @ts-ignore
global.Response = fetch.Response;

const assert = require("assert");

/*
// Test Borsh serialization
(async () => {
    const { getLamports, getOwner, testInstructionBorshWasm, Pubkey } = await import("../pkg");
    try {
        var account = new Pubkey("7z9HJcqrouhUHo3EkbVXRtRxGccJxGGNUYy8AdbseoZa");
        let owner = await getOwner(account);
        let ownerPubkey = new PublicKey(owner.toBytes());
        console.log("AUCTION POOL OWNER:", ownerPubkey.toString());
        assert(ownerPubkey.toString() === "go1dcKcvafq8SDwmBKo6t2NVzyhvTEZJkMwnnfae99U");
        var account = new Pubkey("7z9HJcqrouhUHo3EkbVXRtRxGccJxGGNUYy8AdbseoZa"); // NOTE has to be duplicated, otherwise it becomes a null pointer for some reason
        let lamports = await getLamports(account);
        console.log("AUCTION POOL BALANCE:", Number(lamports));
    } catch (error) {
        console.log(error);
    }
    const serialized_input = Uint8Array.from([
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 1, 15, 0, 0, 0
    ]);

    try {
        const instruction = JSON.parse(testInstructionBorshWasm(serialized_input));
        assert(new PublicKey(instruction.program_id).toString() === "PwDiXFxQsGra4sFFTT8r1QWRMd4vfumiWC1jfWNfdYT");
        console.log(new PublicKey(instruction.accounts[0].pubkey).toString());
        assert(new PublicKey(instruction.accounts[0].pubkey).toString() === "4uQeVj5tqViQh7yWWGStvkEG1Zmhx6uasJtWCJziofM");
        assert(new PublicKey(instruction.accounts[1].pubkey).toString() === "HKp9TzCTQ79TE4eppvHWUUXVaZePZSJCYkExtEVYjezP");
        assert(new PublicKey(instruction.accounts[2].pubkey).toString() === "6UUakecVHBoXBxh6sbQd9mEx6yikDQ9cy1f7jobcyucc");
        assert(instruction.accounts[0].is_writable);
        assert(instruction.accounts[0].is_signer);
        assert(instruction.accounts[1].is_writable);
        assert(!instruction.accounts[1].is_signer);
        assert(!instruction.accounts[2].is_writable);
        assert(!instruction.accounts[2].is_signer);
        assert(JSON.stringify(instruction.data) === JSON.stringify([0, 1, 15, 0, 0, 0]));
        console.log(instruction);
    } catch (error) {
        console.log(error);
        assert(false);
    }
}) ()
*/


// Test Serde serialization
type FrontendTestInstructionArgs = {
    pubkey: PublicKey,
    input: number,
};

(async () => {
    const { getLamports, getOwner, testInstructionSerdeWasm, Pubkey } = await import("../pkg");
    try {
        var account = new Pubkey("7z9HJcqrouhUHo3EkbVXRtRxGccJxGGNUYy8AdbseoZa");
        let owner = await getOwner(account);
        let ownerPubkey = new PublicKey(owner.toBytes());
        console.log("AUCTION POOL OWNER:", ownerPubkey.toString());
        assert(ownerPubkey.toString() === "go1dcKcvafq8SDwmBKo6t2NVzyhvTEZJkMwnnfae99U");
        var account = new Pubkey("7z9HJcqrouhUHo3EkbVXRtRxGccJxGGNUYy8AdbseoZa"); // NOTE has to be duplicated, otherwise it becomes a null pointer for some reason
        let lamports = await getLamports(account);
        console.log("AUCTION POOL BALANCE:", Number(lamports));
    } catch (error) {
        console.log(error);
    }
    const obj_input = {
        pubkey: new PublicKey('4uQeVj5tqViQh7yWWGStvkEG1Zmhx6uasJtWCJziofM'),
        input: 15,
    };

    try {
        const instruction = testInstructionSerdeWasm(obj_input);
        assert(new PublicKey(instruction.program_id).toString() === "PwDiXFxQsGra4sFFTT8r1QWRMd4vfumiWC1jfWNfdYT");
        console.log(new PublicKey(instruction.accounts[0].pubkey).toString());
        assert(new PublicKey(instruction.accounts[0].pubkey).toString() === "4uQeVj5tqViQh7yWWGStvkEG1Zmhx6uasJtWCJziofM");
        assert(new PublicKey(instruction.accounts[1].pubkey).toString() === "HKp9TzCTQ79TE4eppvHWUUXVaZePZSJCYkExtEVYjezP");
        assert(new PublicKey(instruction.accounts[2].pubkey).toString() === "6UUakecVHBoXBxh6sbQd9mEx6yikDQ9cy1f7jobcyucc");
        assert(instruction.accounts[0].is_writable);
        assert(instruction.accounts[0].is_signer);
        assert(instruction.accounts[1].is_writable);
        assert(!instruction.accounts[1].is_signer);
        assert(!instruction.accounts[2].is_writable);
        assert(!instruction.accounts[2].is_signer);
        assert(JSON.stringify(instruction.data) === JSON.stringify([0, 1, 15, 0, 0, 0]));
        console.log(instruction);
    } catch (error) {
        console.log(error);
        assert(false);
    }
}) ()

