const path				= require('path');
const log				= require('@whi/stdlog')(path.basename( __filename ), {
    level: process.env.LOG_LEVEL || 'fatal',
});


const fs				= require('fs');
const crypto				= require('crypto');
const expect				= require('chai').expect;
// const why				= require('why-is-node-running');

const msgpack				= require('@msgpack/msgpack');
const json				= require('@whi/json');
const { ActionHash, AgentPubKey,
	HoloHash }			= require('@whi/holo-hash');
const { Holochain }			= require('@whi/holochain-backdrop');
const { CruxConfig }			= require('@whi/crux-payload-parser');
const { ConductorError,
	...hc_client }			= require('@whi/holochain-client');

const { expect_reject }			= require('../utils.js');

const delay				= (n) => new Promise(f => setTimeout(f, n));

const PORTAL_DNA_PATH			= path.join( __dirname, "../../bundled/portal.dna" );

const clients				= {};


function host_tests () {
    let host_1;

    it("should register hosts", async function () {
	const host			= await clients.alice.call("portal", "portal_api", "register_host", {
	    "dna": "uhC0kXracwD-PyrSU5m_unW3GA7vV1fY1eHH-0qV5HG7Y7s-DwLa5",
	    "granted_functions": {
		"Listed": [
		    [ "testing", "testing" ],
		],
	    },
	});

	await clients.bobby.call("portal", "portal_api", "register_host", {
	    "dna": "uhC0kXracwD-PyrSU5m_unW3GA7vV1fY1eHH-0qV5HG7Y7s-DwLa5",
	    "granted_functions": {
		"Listed": [
		    [ "testing", "testing" ],
		],
	    },
	});
    });

    it("should get registered hosts", async function () {
	const hosts			= await clients.alice.call("portal", "portal_api", "get_registered_hosts", {
	    "dna": "uhC0kXracwD-PyrSU5m_unW3GA7vV1fY1eHH-0qV5HG7Y7s-DwLa5",
	});

	expect( hosts			).to.have.length( 2 );
    });

    it("should ping host", async function () {
	const resp			= await clients.alice.call("portal", "portal_api", "ping", clients.bobby.cellAgent() );

	expect( resp			).to.be.true;
    });

}

function errors_tests () {
}

describe("Portal", () => {
    const crux				= new CruxConfig();
    const holochain			= new Holochain({
	"default_stdout_loggers": process.env.LOG_LEVEL === "silly",
    });

    before(async function () {
	this.timeout( 60_000 );

	const actors			= await holochain.backdrop({
	    "test_happ": {
		"portal":	PORTAL_DNA_PATH,
	    },
	}, {
	    "actors": [
		"alice",
		"bobby",
	    ],
	});

	for ( let name in actors ) {
	    for ( let app_prefix in actors[ name ] ) {
		log.info("Upgrade client for %s => %s", name, app_prefix );
		const client		= clients[ name ]	= actors[ name ][ app_prefix ].client;

		crux.upgrade( client );
	    }
	}

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
