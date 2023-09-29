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
const { CruxConfig }			= require('@whi/crux-payload-parser');
const { Holochain,
	HolochainClientLib }		= require('@whi/holochain-backdrop');
const { ConductorError,
	...hc_client }			= HolochainClientLib;

const { expect_reject }			= require('../utils.js');

const delay				= (n) => new Promise(f => setTimeout(f, n));

const APPSTORE_DNA_PATH			= path.join( __dirname, "../../bundled/appstore.dna" );
const TEST_DNA_HASH			= "uhC0kXracwD-PyrSU5m_unW3GA7vV1fY1eHH-0qV5HG7Y7s-DwLa5";

const clients				= {};


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
	"icon": crypto.randomBytes(1_000),
	"email": "techservices@holo.host",
	"editors": [
	    new AgentPubKey( crypto.randomBytes(32) )
	],
    }, overrides );
};


let publisher_1;

function publisher_tests () {

    it("should create publisher profile", async function () {
	this.timeout( 10_000 );

	const publisher = publisher_1	= await clients.alice.call("appstore", "appstore_api", "create_publisher", createPublisherInput() );

	// log.debug( json.debug( publisher ) );

	expect( publisher.editors	).to.have.length( 2 );
    });

    it("should get publisher profile", async function () {
	const publisher			= await clients.alice.call("appstore", "appstore_api", "get_publisher", {
	    "id": publisher_1.$id,
	});

	expect( publisher.$id		).to.deep.equal( publisher_1.$id );
    });

    it("should get publishers for an agent", async function () {
	const publishers		= await clients.alice.call("appstore", "appstore_api", "get_publishers_for_agent", {
	    "for_agent": clients.alice.cellAgent(),
	});

	expect( publishers		).to.have.length( 1 );
    });

    it("should get my publishers", async function () {
	const publishers		= await clients.alice.call("appstore", "appstore_api", "get_my_publishers");

	expect( publishers		).to.have.length( 1 );
    });

    it("should update publisher profile", async function () {
	const publisher = publisher_1	= await clients.alice.call("appstore", "appstore_api", "update_publisher", {
	    "base": publisher_1.$action,
	    "properties": {
		"name": "Holo Inc",
	    },
	});

	expect( publisher.name		).to.equal( "Holo Inc" );
    });

    it("should get all publishers", async function () {
	const publishers		= await clients.alice.call("appstore", "appstore_api", "get_all_publishers");

	expect( publishers		).to.have.length( 1 );
    });

    it("should deprecate publisher", async function () {
	const publisher		= await clients.alice.call("appstore", "appstore_api", "create_publisher", createPublisherInput() );

	{
	    const publishers	= await clients.alice.call("appstore", "appstore_api", "get_my_publishers");
	    expect( publishers	).to.have.length( 2 );
	}
	{
	    const publishers	= await clients.alice.call("appstore", "appstore_api", "get_all_publishers");
	    expect( publishers	).to.have.length( 2 );
	}

	await clients.alice.call("appstore", "appstore_api", "deprecate_publisher", {
	    "base": publisher.$action,
	    "message": "Oopsie!",
	});

	{
	    const publishers	= await clients.alice.call("appstore", "appstore_api", "get_my_publishers");
	    expect( publishers	).to.have.length( 2 );
	}
	{
	    const publishers	= await clients.alice.call("appstore", "appstore_api", "get_all_publishers");
	    expect( publishers	).to.have.length( 1 );
	}
    });

}


function createAppInput ( overrides ) {
    return Object.assign({
	"title": "Chess",
	"subtitle": "The classic boardgame",
	"description": "The boardgame known as Chess",
	"icon": crypto.randomBytes(1_000),
	"publisher": publisher_1.$id,
	"devhub_address": {
	    "dna": TEST_DNA_HASH,
	    "happ": publisher_1.$id,
	    "gui": publisher_1.$id,
	},
	"editors": [
	    new AgentPubKey( crypto.randomBytes(32) )
	],
    }, overrides );
};


let app_1;

function app_tests () {

    it("should create app profile", async function () {
	this.timeout( 10_000 );

	const app = app_1		= await clients.alice.call("appstore", "appstore_api", "create_app", createAppInput() );

	// log.debug( json.debug( app ) );

	expect( app.editors		).to.have.length( 2 );
    });

    it("should get app profile", async function () {
	const app			= await clients.alice.call("appstore", "appstore_api", "get_app", {
	    "id": app_1.$id,
	});

	expect( app.$id			).to.deep.equal( app_1.$id );
    });

    it("should get apps for an agent", async function () {
	const apps		= await clients.alice.call("appstore", "appstore_api", "get_apps_for_agent", {
	    "for_agent": clients.alice.cellAgent(),
	});

	expect( apps		).to.have.length( 1 );
    });

    it("should get my apps", async function () {
	const apps		= await clients.alice.call("appstore", "appstore_api", "get_my_apps");

	expect( apps		).to.have.length( 1 );
    });

    it("should get all apps", async function () {
	const apps		= await clients.alice.call("appstore", "appstore_api", "get_all_apps");

	expect( apps		).to.have.length( 1 );
    });

    it("should deprecate app", async function () {
	const app		= await clients.alice.call("appstore", "appstore_api", "create_app", createAppInput() );

	{
	    const apps		= await clients.alice.call("appstore", "appstore_api", "get_my_apps");
	    expect( apps	).to.have.length( 2 );
	}
	{
	    const apps		= await clients.alice.call("appstore", "appstore_api", "get_all_apps");
	    expect( apps	).to.have.length( 2 );
	}

	await clients.alice.call("appstore", "appstore_api", "deprecate_app", {
	    "base": app.$action,
	    "message": "Oopsie!",
	});

	{
	    const apps		= await clients.alice.call("appstore", "appstore_api", "get_my_apps");
	    expect( apps	).to.have.length( 2 );
	}
	{
	    const apps		= await clients.alice.call("appstore", "appstore_api", "get_all_apps");
	    expect( apps	).to.have.length( 1 );
	}
    });

}


const ICON_SIZE_LIMIT		= 204_800;

function errors_tests () {
    it("should fail to update publisher because bad author", async function () {
	this.timeout( 10_000 );

	await expect_reject( async () => {
	    await clients.bobby.call("appstore", "appstore_api", "update_publisher", {
		"base": publisher_1.$action,
		"properties": {
		    "name": "Malicious",
		},
	    });
	}, ConductorError, "InvalidCommit error: Previous entry author does not match Action author" );
    });

    it("should fail to update app because bad author", async function () {
	this.timeout( 10_000 );

	await expect_reject( async () => {
	    await clients.bobby.call("appstore", "appstore_api", "update_app", {
		"base": app_1.$action,
		"properties": {
		    "name": "Malicious",
		},
	    });
	}, ConductorError, "InvalidCommit error: Previous entry author does not match Action author" );
    });

    it("should fail to create publisher because icon is too big", async function () {
	this.timeout( 10_000 );

	await expect_reject( async () => {
	    const input			= createPublisherInput();
	    input.icon			= new Uint8Array( ICON_SIZE_LIMIT + 1 ).fill(0);
	    await clients.alice.call("appstore", "appstore_api", "create_publisher", input );
	}, ConductorError, `InvalidCommit error: PublisherEntry icon cannot be larger than ${Math.floor(ICON_SIZE_LIMIT/1024)}KB (${ICON_SIZE_LIMIT} bytes)` );
    });

    it("should fail to update publisher because icon is too big", async function () {
	this.timeout( 10_000 );

	await expect_reject( async () => {
	    const memory_addr		= await clients.alice.call("appstore", "mere_memory_api", "save_bytes", new Uint8Array( ICON_SIZE_LIMIT + 1 ).fill(0) );
	    await clients.alice.call("appstore", "appstore_api", "update_publisher", {
		"base": publisher_1.$action,
		"properties": {
		    "icon": memory_addr,
		},
	    });
	}, ConductorError, `InvalidCommit error: PublisherEntry icon cannot be larger than ${Math.floor(ICON_SIZE_LIMIT/1024)}KB (${ICON_SIZE_LIMIT} bytes)` );
    });

    it("should fail to create app because icon is too big", async function () {
	this.timeout( 10_000 );

	await expect_reject( async () => {
	    const input			= createAppInput();
	    input.icon			= new Uint8Array( ICON_SIZE_LIMIT + 1 ).fill(0);
	    await clients.alice.call("appstore", "appstore_api", "create_app", input );
	}, ConductorError, `InvalidCommit error: AppEntry icon cannot be larger than ${Math.floor(ICON_SIZE_LIMIT/1024)}KB (${ICON_SIZE_LIMIT} bytes)` );
    });

    it("should fail to update app because icon is too big", async function () {
	this.timeout( 10_000 );

	await expect_reject( async () => {
	    const memory_addr		= await clients.alice.call("appstore", "mere_memory_api", "save_bytes", new Uint8Array( ICON_SIZE_LIMIT + 1 ).fill(0) );
	    await clients.alice.call("appstore", "appstore_api", "update_app", {
		"base": app_1.$action,
		"properties": {
		    "icon": memory_addr,
		},
	    });
	}, ConductorError, `InvalidCommit error: AppEntry icon cannot be larger than ${Math.floor(ICON_SIZE_LIMIT/1024)}KB (${ICON_SIZE_LIMIT} bytes)` );
    });
}

describe("Appstore", () => {
    const crux				= new CruxConfig();
    const holochain			= new Holochain({
	"timeout": 60_000,
	"default_stdout_loggers": process.env.LOG_LEVEL === "silly",
    });

    before(async function () {
	this.timeout( 60_000 );

	const actors			= await holochain.backdrop({
	    "test_happ": {
		"appstore":	APPSTORE_DNA_PATH,
	    },
	}, {
	    "actors": [ "alice", "bobby" ],
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
	    let whoami			= await clients.alice.call( "appstore", "appstore_api", "whoami", null, 30_000 );
	    log.normal("Alice whoami: %s", String(new HoloHash( whoami.agent_initial_pubkey )) );
	}
	{
	    let whoami			= await clients.bobby.call( "appstore", "appstore_api", "whoami", null, 30_000 );
	    log.normal("Bobby whoami: %s", String(new HoloHash( whoami.agent_initial_pubkey )) );
	}
    });

    describe("Publisher", publisher_tests.bind( this, holochain ) );
    describe("App", app_tests.bind( this, holochain ) );
    describe("Errors", errors_tests.bind( this, holochain ) );

    after(async () => {
	await holochain.destroy();
    });

});
