import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Magikmons } from "../target/types/magikmons";
import { PublicKey, Keypair } from "@solana/web3.js";
import { expect } from "chai";

describe("Magikmons Phase 2", () => {
    const provider=anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program=anchor.workspace.Magikmons as Program<Magikmons>;

    let authority: Keypair;
    let testKeypair: Keypair;
    let gameConfig: PublicKey;
    let playerAccount: PublicKey;
    let battleState: PublicKey;
    let treasury: PublicKey;

    before(async () => {
        // will updated authority to turbin3 wallet
        authority=Keypair.generate();
        testKeypair=Keypair.generate();
        // treasury will be one of the dev wallet address
        treasury=Keypair.generate();


        const airdropTx1=await provider.connection.requestAirdrop(
            authority.publicKey,
            1*anchor.web3.LAMPORTS_PER_SOL
        );
        await provider.connection.confirmTransaction(airdropTx1);

        const airdropTx2=await provider.connection.requestAirdrop(
            testKeypair.publicKey,
            1*anchor.web3.LAMPORTS_PER_SOL
        );
        await provider.connection.confirmTransaction(airdropTx2);


        [gameConfig]=anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("config")],
            program.programId
        );

        [playerAccount]=anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("player"), testKeypair.publicKey.toBuffer()],
            program.programId
        );

        console.log("Test setup complete:");
        console.log("Authority:", authority.publicKey.toString());
        console.log("Test keypair:", testKeypair.publicKey.toString());
        console.log("Game config:", gameConfig.toString());
        console.log("Player account:", playerAccount.toString());
        console.log("Treasury:", treasury.publicKey.toString());
    });

    it("Initialize game with default NPCs", async () => {
        const tx=await program.methods
            .initializeGame(treasury.publicKey)
            .accountsPartial({
                authority: authority.publicKey,
                gameConfig,
                systemProgram: anchor.web3.SystemProgram.programId,
            })
            .signers([authority])
            .rpc();

        console.log("Initialize Game Transaction Signature:", tx);

        const gameConfigData=await program.account.gameConfig.fetch(gameConfig);
        console.log("GAME CONFIG DATA: ", gameConfigData)

        expect(gameConfigData.authority.toString()).to.equal(authority.publicKey.toString());
        expect(gameConfigData.treasury.toString()).to.equal(treasury.publicKey.toString());
        expect(gameConfigData.totalCities).to.equal(1);
        expect(gameConfigData.npcConfigs).to.have.lengthOf(3); // 

        const npc1=gameConfigData.npcConfigs[0];
        expect(npc1.name).to.equal("Natty Node Nate");
        expect(npc1.monsters).to.deep.equal(["f001"]);
        expect(npc1.monsterLevels[0]).to.equal(1);

        const npc2=gameConfigData.npcConfigs[1];
        expect(npc2.name).to.equal("Liquidity Lord Andre");
        expect(npc2.monsters).to.deep.equal(["w001", "f002"]);
        expect(npc2.monsterLevels).to.have.lengthOf(2);
        expect(npc2.monsterLevels[0]).to.equal(2);
        expect(npc2.monsterLevels[1]).to.equal(2);

        const npc3=gameConfigData.npcConfigs[2];
        expect(npc3.name).to.equal("Devnet Whale Jeff");
        expect(npc3.monsters).to.deep.equal(["l001", "f003", "w002"]);
        expect(npc3.monsterLevels).to.have.lengthOf(3);
        expect(npc3.monsterLevels[0]).to.equal(3);
        expect(npc3.monsterLevels[1]).to.equal(3);
        expect(npc3.monsterLevels[2]).to.equal(4);

        console.log("Game initialized successfully with default NPCs");
    });

    it("Create player with starter monster", async () => {
        const playerName="TestTrainer";

        const tx=await program.methods
            .createPlayer(playerName)
            .accountsPartial({
                signer: testKeypair.publicKey,
                playerAccount,
                systemProgram: anchor.web3.SystemProgram.programId,
            })
            .signers([testKeypair])
            .rpc();

        console.log("Create Player Transaction Signature:", tx);

        const playerAccountData=await program.account.playerAccount.fetch(playerAccount);

        expect(playerAccountData.owner.toString()).to.equal(testKeypair.publicKey.toString());
        expect(playerAccountData.name).to.equal(playerName);
        expect(playerAccountData.level).to.equal(1);
        expect(playerAccountData.currentXp).to.equal(0);
        expect(playerAccountData.maxXp).to.equal(100);
        expect(playerAccountData.defeatedNpcs).to.have.lengthOf(20);
        expect(playerAccountData.defeatedNpcs.every(defeated => !defeated)).to.be.true;
        expect(playerAccountData.monsters).to.have.lengthOf(1);
        expect(playerAccountData.activeLineup[0]).to.equal(0);
        expect(playerAccountData.items).to.have.lengthOf(1);
        expect(playerAccountData.badges).to.have.lengthOf(0);
        expect(playerAccountData.totalBattles).to.equal(0);
        expect(playerAccountData.battlesWon).to.equal(0);


        const starter=playerAccountData.monsters[0];
        expect(starter.monsterId).to.equal("f001");
        expect(starter.level).to.equal(1);
        expect(starter.currentHp).to.equal(50);
        expect(starter.maxHp).to.equal(50);
        expect(starter.currentXp).to.equal(0);
        expect(starter.maxXp).to.equal(100);
        expect(starter.moves).to.deep.equal(["damage1"]);
        expect(starter.status).to.be.null;

        console.log("Player created successfully with starter monster");
    });

    it("Should fail to create player with name too long", async () => {
        const longName="A".repeat(35); // Exceeds MAX_NAME_LEN (32)
        const newKeypair=Keypair.generate();


        const airdropSig=await provider.connection.requestAirdrop(
            newKeypair.publicKey,
            0.5*anchor.web3.LAMPORTS_PER_SOL
        );
        await provider.connection.confirmTransaction(airdropSig);

        const [newPlayerAccount]=anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("player"), newKeypair.publicKey.toBuffer()],
            program.programId
        );

        try {
            await program.methods
                .createPlayer(longName)
                .accountsPartial({
                    signer: newKeypair.publicKey,
                    playerAccount: newPlayerAccount,
                    systemProgram: anchor.web3.SystemProgram.programId,
                })
                .signers([newKeypair])
                .rpc();

            expect.fail("Should have failed with name too long error");
        } catch (error) {
            console.log("Correctly failed with name too long error");
            expect(error).to.not.be.undefined;
        }
    });

    it("Start battle against first trainer (NPC ID 0)", async () => {
        const npcId=0;

        [battleState]=anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("battle"), testKeypair.publicKey.toBuffer(), Buffer.from([npcId])],
            program.programId
        );

        const tx=await program.methods
            .startBattle(npcId)
            .accountsPartial({
                signer: testKeypair.publicKey,
                playerAccount,
                gameConfig,
                battleState,
                systemProgram: anchor.web3.SystemProgram.programId,
            })
            .signers([testKeypair])
            .rpc();

        console.log("Start Battle Transaction Signature:", tx);

        const battleStateData=await program.account.battleState.fetch(battleState);


        expect(battleStateData.player.toString()).to.equal(playerAccount.toString());
        expect(battleStateData.npcId).to.equal(npcId);
        expect(battleStateData.activePlayerMonster).to.equal(0);
        expect(battleStateData.activeNpcMonster).to.equal(0);
        expect(battleStateData.currentTurn).to.equal(0);
        expect(battleStateData.playerMonsters).to.have.lengthOf(1);
        expect(battleStateData.npcMonsters).to.have.lengthOf(1);
        expect(battleStateData.turnEvents).to.have.length.greaterThan(0);

        console.log("Battle started successfully against NPC 0");
        console.log("Initial battle events:", battleStateData.turnEvents);
    });

    it("Execute battle turns with different actions", async () => {
        let battleStateData=await program.account.battleState.fetch(battleState);
        let turnCount=0;
        const maxTurns=15;

        console.log("=== EXECUTING BATTLE TURNS ===");

        while (battleStateData.status.active!==undefined&&turnCount<maxTurns) {
            console.log(`\n--- Turn ${turnCount+1} ---`);

            const playerMonster=battleStateData.playerMonsters[battleStateData.activePlayerMonster];
            const npcMonster=battleStateData.npcMonsters[battleStateData.activeNpcMonster];

            console.log(`Player Monster (${playerMonster.monsterId}): ${playerMonster.currentHp}/${playerMonster.maxHp} HP`);
            console.log(`NPC Monster (${npcMonster.monsterId}): ${npcMonster.currentHp}/${npcMonster.maxHp} HP`);

            const playerAction={ attack: ["damage1"] };

            try {
                await new Promise(r => setTimeout(r, 1000));
                console.log("<<<EXECUTING TURN>>>")
                const tx=await program.methods
                    .executeTurn(playerAction)
                    .accountsPartial({
                        signer: testKeypair.publicKey,
                        playerAccount,
                        battleState,
                    })
                    .signers([testKeypair])
                    .rpc();

                console.log(`Turn ${turnCount+1} Transaction:`, tx);

                battleStateData=await program.account.battleState.fetch(battleState);

                const newPlayerMonster=battleStateData.playerMonsters[battleStateData.activePlayerMonster];
                const newNpcMonster=battleStateData.npcMonsters[battleStateData.activeNpcMonster];

                console.log(`After turn - Player HP: ${newPlayerMonster.currentHp}, NPC HP: ${newNpcMonster.currentHp}`);
                console.log("Latest events:", battleStateData.turnEvents.slice(-2));

                if (newNpcMonster.currentHp===0) {
                    console.log("NPC MONSTER DEFEATED!");
                }
                if (newPlayerMonster.currentHp===0) {
                    console.log("PLAYER MONSTER DEFEATED!");
                }

                turnCount++;
            } catch (error) {
                console.log(`Error on turn ${turnCount+1}:`, error.toString());
                break;
            }
        }

        console.log("Battle completed with status:", Object.keys(battleStateData.status)[0]);
    });

    it("End battle and claim rewards", async () => {
        const battleStateData=await program.account.battleState.fetch(battleState);

        if (battleStateData.status.active!==undefined) {
            console.log("Battle is still active, cannot end battle yet");
            console.log("Skipping end battle test");
            return;
        }

        const initialPlayerData=await program.account.playerAccount.fetch(playerAccount);
        const initialXp=initialPlayerData.currentXp;
        const initialBattlesWon=initialPlayerData.battlesWon;
        const initialTotalBattles=initialPlayerData.totalBattles;

        const tx=await program.methods
            .endBattle()
            .accountsPartial({
                signer: testKeypair.publicKey,
                playerAccount,
                gameConfig,
                battleState,
            })
            .signers([testKeypair])
            .rpc();

        console.log("End Battle Transaction Signature:", tx);

        const finalPlayerData=await program.account.playerAccount.fetch(playerAccount);

        if (finalPlayerData.battlesWon>initialBattlesWon) {
            expect(finalPlayerData.defeatedNpcs[0]).to.be.true;
            expect(finalPlayerData.currentXp).to.be.greaterThan(initialXp);
            console.log("Player won! XP gained:", finalPlayerData.currentXp-initialXp);
        } else {
            console.log("Player lost, but all monsters healed");
            expect(finalPlayerData.monsters[0].currentHp).to.equal(finalPlayerData.monsters[0].maxHp);
        }

        expect(finalPlayerData.totalBattles).to.equal(initialTotalBattles+1);
        console.log("Battle ended successfully");
    });

    it("Heal monsters at Pokemon Center", async () => {
        const tx=await program.methods
            .healMonsters()
            .accountsPartial({
                signer: testKeypair.publicKey,
                playerAccount,
            })
            .signers([testKeypair])
            .rpc();

        console.log("Heal Monsters Transaction Signature:", tx);

        const healedPlayerData=await program.account.playerAccount.fetch(playerAccount);

        healedPlayerData.monsters.forEach((monster, index) => {
            expect(monster.currentHp).to.equal(monster.maxHp);
            expect(monster.status).to.be.null;
        });

        console.log("All monsters healed to full HP");
    });

    it("Travel to different cities", async () => {
        const tx1=await program.methods
            .travelToCity({ surfpoolCity: {} })
            .accountsPartial({
                signer: testKeypair.publicKey,
                playerAccount,
            })
            .signers([testKeypair])
            .rpc();

        console.log("Travel to Surf City(Surfpool) Transaction:", tx1);

        let playerData=await program.account.playerAccount.fetch(playerAccount);
        expect(Object.keys(playerData.currentCity)[0]).to.equal("surfpoolCity");

        const tx2=await program.methods
            .travelToCity({ solCity: {} })
            .accountsPartial({
                signer: testKeypair.publicKey,
                playerAccount,
            })
            .signers([testKeypair])
            .rpc();

        console.log("Travel to Sol City(Solana) Transaction:", tx2);

        playerData=await program.account.playerAccount.fetch(playerAccount);
        expect(Object.keys(playerData.currentCity)[0]).to.equal("solCity");

        const tx3=await program.methods
            .travelToCity({ turbineTown: {} })
            .accountsPartial({
                signer: testKeypair.publicKey,
                playerAccount,
            })
            .signers([testKeypair])
            .rpc();

        console.log("Travel back to Turbine Town Transaction:", tx3);

        playerData=await program.account.playerAccount.fetch(playerAccount);
        expect(Object.keys(playerData.currentCity)[0]).to.equal("turbineTown");

        console.log("Travel system works correctly");
    });

    it("Should fail to travel to same city", async () => {
        try {
            await program.methods
                .travelToCity({ turbineTown: {} })
                .accountsPartial({
                    signer: testKeypair.publicKey,
                    playerAccount,
                })
                .signers([testKeypair])
                .rpc();

            expect.fail("Should have failed when trying to travel to same city");
        } catch (error) {
            console.log("Correctly failed when trying to travel to same city");
            expect(error).to.not.be.undefined;
        }
    });

    it("Battle sequence with second trainer if first was defeated", async () => {
        const playerData=await program.account.playerAccount.fetch(playerAccount);

        if (!playerData.defeatedNpcs[0]) {
            console.log("Skipping second battle - first NPC not defeated");
            return;
        }

        const npcId=1;

        [battleState]=anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("battle"), testKeypair.publicKey.toBuffer(), Buffer.from([npcId])],
            program.programId
        );

        try {
            const startTx=await program.methods
                .startBattle(npcId)
                .accountsPartial({
                    signer: testKeypair.publicKey,
                    playerAccount,
                    gameConfig,
                    battleState,
                    systemProgram: anchor.web3.SystemProgram.programId,
                })
                .signers([testKeypair])
                .rpc();

            console.log(`\n=== SECOND BATTLE STARTED vs NPC ${npcId} ===`);
            console.log("Start Battle Transaction Signature:", startTx);

            let battleStateData=await program.account.battleState.fetch(battleState);
            let turnCount=0;

            console.log("NPC Monsters:", battleStateData.npcMonsters);
            console.log("Player Monsters:", battleStateData.playerMonsters);

            while (battleStateData.status.active!==undefined&&turnCount<8) {
                console.log(`\n--- Turn ${turnCount+1} ---`);
                const playerMonster=battleStateData.playerMonsters[battleStateData.activePlayerMonster];
                const npcMonster=battleStateData.npcMonsters[battleStateData.activeNpcMonster];

                console.log(`Player Monster (${playerMonster.monsterId}): ${playerMonster.currentHp}/${playerMonster.maxHp} HP`);
                console.log(`NPC Monster (${npcMonster.monsterId}): ${npcMonster.currentHp}/${npcMonster.maxHp} HP`);

                const playerAction={ attack: ["damage1"] };
                try {
                    console.log("<<< EXECUTING TURN >>>");
                    await new Promise(r => setTimeout(r, 1000));
                    const tx=await program.methods
                        .executeTurn(playerAction)
                        .accountsPartial({
                            signer: testKeypair.publicKey,
                            playerAccount,
                            battleState,
                        })
                        .signers([testKeypair])
                        .rpc();

                    console.log(`Turn ${turnCount+1} Transaction Signature:`, tx);


                    battleStateData=await program.account.battleState.fetch(battleState);

                    const newPlayerMonster=battleStateData.playerMonsters[battleStateData.activePlayerMonster];
                    const newNpcMonster=battleStateData.npcMonsters[battleStateData.activeNpcMonster];

                    console.log(`After turn - Player HP: ${newPlayerMonster.currentHp}, NPC HP: ${newNpcMonster.currentHp}`);
                    console.log("Latest events:", battleStateData.turnEvents.slice(-2));

                    if (newNpcMonster.currentHp===0) {
                        console.log("NPC MONSTER DEFEATED!");
                    }
                    if (newPlayerMonster.currentHp===0) {
                        console.log("PLAYER MONSTER DEFEATED!");
                    }

                    turnCount++;
                } catch (error) {
                    console.log("Turn failed:", error.message);
                    break;
                }
            }

            if (battleStateData.status.active===undefined) {
                console.log("Battle inactive - ending battle...");
                await program.methods
                    .endBattle()
                    .accountsPartial({
                        signer: testKeypair.publicKey,
                        playerAccount,
                        gameConfig,
                        battleState,
                    })
                    .signers([testKeypair])
                    .rpc();
            }

            console.log("\n=== SECOND BATTLE COMPLETED ===");
            console.log("Final battle status:", Object.keys(battleStateData.status)[0]);

        } catch (error) {
            console.log("Second battle failed (expected in some cases):", error.message);
        }
    });

    it("Battle sequence with gym leader after defeating all trainers", async () => {
        const npcId=2;

        const playerData=await program.account.playerAccount.fetch(playerAccount)

        console.log("PLAYER DATA: ", playerData)

        if (!(playerData.defeatedNpcs[0]&&playerData.defeatedNpcs[1])) {
            console.log("Skipping gym leader battle - Trainers not defeated!")
            return;
        }

        [battleState]=anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("battle"), testKeypair.publicKey.toBuffer(), Buffer.from([npcId])],
            program.programId
        );

        try {
            const startTx=await program.methods.startBattle(npcId).accountsPartial({
                signer: testKeypair.publicKey,
                playerAccount,
                gameConfig,
                battleState,
                systemProgram: anchor.web3.SystemProgram.programId
            }).signers([testKeypair]).rpc();

            console.log(`\n=== GYM LEADER BATTLE STARTED (NPC ${npcId}) ===`);
            console.log("Start Battle Transaction Signature:", startTx);

            let battleStateData=await program.account.battleState.fetch(battleState);
            let turnCount=0;
            let maxTurns=30;

            while (battleStateData.status.active!==undefined&&turnCount<maxTurns) {
                console.log(`\n--- Turn ${turnCount+1} ---`);
                const playerMonster=battleStateData.playerMonsters[battleStateData.activePlayerMonster];
                const npcMonster=battleStateData.npcMonsters[battleStateData.activeNpcMonster];
                console.log(`Player Monster (${playerMonster.monsterId}): ${playerMonster.currentHp}/${playerMonster.maxHp} HP`);
                console.log(`Gym Leader Monster (${npcMonster.monsterId}): ${npcMonster.currentHp}/${npcMonster.maxHp} HP`);

                const playerAction={ attack: ["damage1"] };
                try {
                    console.log("<<< EXECUTING TURN >>>");
                    await new Promise(r => setTimeout(r, 1000));
                    const tx=await program.methods
                        .executeTurn(playerAction)
                        .accountsPartial({
                            signer: testKeypair.publicKey,
                            playerAccount,
                            battleState,
                        })
                        .signers([testKeypair])
                        .rpc();

                    console.log(`Turn ${turnCount+1} Transaction Signature:`, tx);

                    battleStateData=await program.account.battleState.fetch(battleState);
                    turnCount++;
                } catch (error) {
                    console.log("Turn failed:", error.message);
                    break;
                }
            }

            if (battleStateData.status.active===undefined) {
                console.log("Battle finished - ending battle...");
                await program.methods
                    .endBattle()
                    .accountsPartial({
                        signer: testKeypair.publicKey,
                        playerAccount,
                        gameConfig,
                        battleState,
                    })
                    .signers([testKeypair])
                    .rpc();
            }

            console.log("\n=== GYM LEADER BATTLE COMPLETED ===");
            console.log("Final battle status:", Object.keys(battleStateData.status)[0]);
        } catch (error) {
            console.log("Gym leader battle failed:", error.message);
        }
    })

    it("Add new NPC to game config (authority only)", async () => {
        const npcConfig={
            city: { turbineTown: {} },
            opponentType: { trainer: {} },
            name: "Gasless Guru Shrinath",
            monsters: ["w003", "l002"],
            monsterLevels: Buffer.from([3, 4])
        };

        const tx=await program.methods
            .addNpcToConfig(npcConfig)
            .accounts({
                authority: authority.publicKey,
                gameConfig,
            })
            .signers([authority])
            .rpc();

        console.log("NPC Added Tx:", tx);

        const gameConfigData=await program.account.gameConfig.fetch(gameConfig);
        const latestNpc=gameConfigData.npcConfigs[gameConfigData.npcConfigs.length-1];

        console.log("GAME CONFIG DATA: ", gameConfigData);
        console.log("latestNpc : ", latestNpc);

        expect(latestNpc.name).to.equal("Gasless Guru Shrinath");
        expect(latestNpc.monsters).to.deep.equal(["w003", "l002"]);
        expect(Array.from(latestNpc.monsterLevels)).to.deep.equal([3, 4]);
    });

    it("Should fail to challenge gym leader without defeating all trainers", async () => {
        const npcId=2;

        [battleState]=anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("battle"), testKeypair.publicKey.toBuffer(), Buffer.from([npcId])],
            program.programId
        );

        const playerData=await program.account.playerAccount.fetch(playerAccount);

        const allTrainersDefeated=playerData.defeatedNpcs[0]&&playerData.defeatedNpcs[1];

        if (allTrainersDefeated) {
            console.log("All trainers defeated - skipping gym leader restriction test");
            return;
        }

        try {
            await program.methods
                .startBattle(npcId)
                .accountsPartial({
                    signer: testKeypair.publicKey,
                    playerAccount,
                    gameConfig,
                    battleState,
                    systemProgram: anchor.web3.SystemProgram.programId,
                })
                .signers([testKeypair])
                .rpc();

            expect.fail("Should have failed - must defeat trainers first");
        } catch (error) {
            console.log("Correctly failed when trying to challenge gym leader without defeating trainers");
            expect(error).to.not.be.undefined;
        }
    });

    it("Display final game state", async () => {
        const finalPlayerData=await program.account.playerAccount.fetch(playerAccount);
        const gameConfigData=await program.account.gameConfig.fetch(gameConfig);

        console.log("=== FINAL GAME STATE ===");
        console.log("Player Name:", finalPlayerData.name);
        console.log("Player Level:", finalPlayerData.level);
        console.log("Player XP:", `${finalPlayerData.currentXp}/${finalPlayerData.maxXp}`);
        console.log("Current City:", Object.keys(finalPlayerData.currentCity)[0]);
        console.log("Total Battles:", finalPlayerData.totalBattles);
        console.log("Battles Won:", finalPlayerData.battlesWon);
        console.log("Badges:", finalPlayerData.badges);

        console.log("\n--- Monsters ---");
        finalPlayerData.monsters.forEach((monster, index) => {
            console.log(`${index}: ${monster.monsterId} (Level ${monster.level})`);
            console.log(`   HP: ${monster.currentHp}/${monster.maxHp}`);
            console.log(`   XP: ${monster.currentXp}/${monster.maxXp}`);
            console.log(`   Moves: ${monster.moves.join(", ")}`);
            if (monster.status) {
                console.log(`   Status: ${monster.status.statusType} (${monster.status.expiresIn} turns)`);
            }
        });

        console.log("\n--- Items ---");
        finalPlayerData.items.forEach(item => {
            console.log(`${item.actionId}: ${item.quantity}`);
        });

        console.log("\n--- Defeated NPCs ---");
        finalPlayerData.defeatedNpcs.forEach((defeated, index) => {
            if (defeated&&index<gameConfigData.npcConfigs.length) {
                console.log(`${index}: ${gameConfigData.npcConfigs[index].name} ✓`);
            }
        });

        console.log("\n--- Available NPCs ---");
        gameConfigData.npcConfigs.forEach((npc, index) => {
            const status=finalPlayerData.defeatedNpcs[index]? "DEFEATED":"Available";
            const city=Object.keys(npc.city)[0];
            const type=Object.keys(npc.opponentType)[0];
            console.log(`${index}: ${npc.name} (${city} ${type}) - ${status}`);
        });

        expect(finalPlayerData.totalBattles).to.be.greaterThanOrEqual(0);
        console.log(`\nGame progress: ${finalPlayerData.battlesWon}/${finalPlayerData.totalBattles} battles won`);
    });

    it("Should fail with invalid NPC ID", async () => {
        const invalidNpcId=99;

        [battleState]=anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("battle"), testKeypair.publicKey.toBuffer(), Buffer.from([invalidNpcId])],
            program.programId
        );

        try {
            await program.methods
                .startBattle(invalidNpcId)
                .accountsPartial({
                    signer: testKeypair.publicKey,
                    playerAccount,
                    gameConfig,
                    battleState,
                    systemProgram: anchor.web3.SystemProgram.programId,
                })
                .signers([testKeypair])
                .rpc();

            expect.fail("Should have failed with invalid NPC ID");
        } catch (error) {
            console.log("Correctly failed with invalid NPC ID");
            expect(error).to.not.be.undefined;
        }
    });
});