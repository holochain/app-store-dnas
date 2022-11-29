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

const APPSTORE_DNA_PATH			= path.join( __dirname, "../../bundled/appstore.dna" );

let clients;

function createPublisherInput ( overrides ) {
    return Object.assign({
	"name": "Holo",
	"location": {
	    "country": "Gibraltar",
	    "region": "Gibraltar",
	    "city": "Gibraltar",
	},
	"website": {
	    "url": "https://github.com/holo-host",
	    "context": "github",
	},
	"icon": new EntryHash( crypto.randomBytes(32) ),
	"email": "techservices@holo.host",
	"editors": [
	    new AgentPubKey( crypto.randomBytes(32) )
	],
    }, overrides );
};


function publisher_tests () {
    let publisher_1;

    it("should create publisher profile", async function () {
	this.timeout( 10_000 );

	const publisher = publisher_1	= await clients.alice.call("appstore", "appstore_api", "create_publisher", createPublisherInput() );

	console.log( json.debug( publisher ) );

	expect( publisher.editors	).to.have.length( 2 );
    });

    it("should get publisher profile", async function () {
	const publisher			= await clients.alice.call("appstore", "appstore_api", "get_publisher", {
	    "id": publisher_1.$id,
	});

	expect( publisher.$id		).to.deep.equal( publisher_1.$id );
    });

    it("should update publisher profile", async function () {
	const publisher = publisher_1	= await clients.alice.call("appstore", "appstore_api", "update_publisher", {
	    "action": publisher_1.$action,
	    "properties": {
		"name": "Holo Inc",
	    },
	});

	expect( publisher.name		).to.equal( "Holo Inc" );
    });

    it("should get publisher profile via remote call", async function () {
	const input			= {
	    "agents": [ clients.alice._agent ],
	    "zome": "appstore_api",
	    "function": "get_publisher",
	    "payload": {
		"id": publisher_1.$id,
	    },
	};
	const publisher			= await clients.bobby.call("appstore", "portal_api", "remote_call", input );

	expect( publisher.$id		).to.deep.equal( publisher_1.$id );
    });

}

// function host_tests () {
//     let host_1;

//     it("should register host", async function () {
// 	const host			= await clients.bobby.call("appstore", "portal_api", "register_host", {
// 	    "dna": "devhub",
// 	    "zome": "dna_library",
// 	    "function": "get_webhapp_package",
// 	});
//     });

// }


function errors_tests () {

    // it("should fail to create Publisher because ", async function () {
    // 	await expect_reject( async () => {
    // 	    await clients.alice.call("appstore", "appstore_api", "create_publisher", createPublisherInput({
    // 		""
    // 	    }) );
    // 	}, ConductorError, "It broke" );

    // });

    // it("should fail to update another Agent's zome", async function () {
    // 	await expect_reject( async () => {
    // 	    throw new TypeError(`Not implemented`);
    // 	}, ConductorError, "It broke" );

    // });

}

describe("DNArepo", () => {

    const holochain			= new Holochain({
	"default_stdout_loggers": process.env.LOG_LEVEL === "silly",
    });

    before(async function () {
	this.timeout( 60_000 );

	clients				= await backdrop( holochain, {
	    "appstore": APPSTORE_DNA_PATH,
	}, [
	    "alice",
	    "bobby",
	]);

	// Must call whoami on each cell to ensure that init has finished.
	{
	    let whoami			= await clients.alice.call( "appstore", "appstore_api", "whoami", null, 30_000 );
	    log.normal("Alice whoami: %s", String(new HoloHash( whoami.agent_initial_pubkey )) );
	}
    });

    describe("Publisher", publisher_tests.bind( this, holochain ) );
    // describe("Host", host_tests.bind( this, holochain ) );
    describe("Errors", errors_tests.bind( this, holochain ) );

    after(async () => {
	await holochain.destroy();
    });

});
