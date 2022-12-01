const path				= require('path');
const log				= require('@whi/stdlog')(path.basename( __filename ), {
    level: process.env.LOG_LEVEL || 'fatal',
});


const fs				= require('fs');
const crypto				= require('crypto');
const expect				= require('chai').expect;
const msgpack				= require('@msgpack/msgpack');
const { EntryHash, AgentPubKey,
	HoloHash }			= require('@whi/holo-hash');
const { Holochain }			= require('@whi/holochain-backdrop');
const json				= require('@whi/json');
// const why				= require('why-is-node-running');
const { ConductorError,
	...hc_client }			= require('@whi/holochain-client');

const { expect_reject }			= require('./utils.js');
const { backdrop }			= require('./setup.js');

const delay				= (n) => new Promise(f => setTimeout(f, n));

const PORTAL_DNA_PATH			= path.join( __dirname, "../../bundled/portal.dna" );

let clients;

function host_tests () {
    let host_1;

    it("should register host", async function () {
	const host			= await clients.alice.call("portal", "portal_api", "register_host", {
	    "dna": "uhC0kXracwD-PyrSU5m_unW3GA7vV1fY1eHH-0qV5HG7Y7s-DwLa5",
	    "zome": "testing",
	    "function": "testing",
	});
    });

    // it("should get publisher profile via remote call", async function () {
    // 	this.timeout( 15_000 );

    // 	const input			= {
    // 	    "dna": DEVHUB_DNA_HASH,
    // 	    "zome": "dna_library",
    // 	    "function": "create_zome",
    // 	    "payload": {
    // 		"name": "New Zome",
    // 		"description": "Testing portal",
    // 	    },
    // 	};
    // 	const zome			= await clients.bobby.call("portal", "portal_api", "remote_call", input );

    // 	console.log( zome );
    // });

}

function errors_tests () {
}

describe("Portal", () => {

    const holochain			= new Holochain({
	"default_stdout_loggers": process.env.LOG_LEVEL === "silly",
    });

    before(async function () {
	this.timeout( 60_000 );

	clients				= await backdrop( holochain, {
	    "portal": PORTAL_DNA_PATH,
	}, [
	    "alice",
	]);

	// Must call whoami on each cell to ensure that init has finished.
	{
	    let whoami			= await clients.alice.call( "portal", "portal_api", "whoami", null, 30_000 );
	    log.normal("Alice whoami: %s", String(new HoloHash( whoami.agent_initial_pubkey )) );
	}
    });

    describe("Host", host_tests.bind( this, holochain ) );
    describe("Errors", errors_tests.bind( this, holochain ) );

    after(async () => {
	await holochain.destroy();
    });

});
