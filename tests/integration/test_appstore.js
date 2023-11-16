import { Logger }			from '@whi/weblogger';
const log				= new Logger("test-app-store", process.env.LOG_LEVEL );

import path				from 'path';
import crypto				from 'crypto';
import { expect }			from 'chai';
// import why				from 'why-is-node-running';

import json				from '@whi/json';
import {
    HoloHash,
    DnaHash, AgentPubKey,
    ActionHash, EntryHash,
}					from '@spartan-hc/holo-hash';
import CruxPayloadParser		from '@whi/crux-payload-parser';
const { CruxConfig }			= CruxPayloadParser;

import HolochainBackdrop		from '@spartan-hc/holochain-backdrop';
const { Holochain }			= HolochainBackdrop;

import { MereMemoryZomelet }		from '@spartan-hc/mere-memory-zomelets';
import {
    AppInterfaceClient,
}					from '@spartan-hc/app-interface-client';

import {
    expect_reject,
    // linearSuite,
    createAppInput,
    createPublisherInput,
}					from '../utils.js';


const delay				= (n) => new Promise(f => setTimeout(f, n));

const __dirname				= path.dirname( new URL(import.meta.url).pathname );
const APPSTORE_DNA_PATH			= path.join( __dirname, "../../bundled/appstore.dna" );
const TEST_DNA_HASH			= "uhC0kXracwD-PyrSU5m_unW3GA7vV1fY1eHH-0qV5HG7Y7s-DwLa5";
const APP_PORT				= 23_567;

const clients				= {};
let client;
let app_client;
let alice_mm;


describe("Appstore", () => {
    const crux				= new CruxConfig();
    const holochain			= new Holochain({
	"timeout": 60_000,
	"default_stdout_loggers": process.env.LOG_LEVEL === "silly",
    });

    before(async function () {
	this.timeout( 60_000 );

	const actors			= await holochain.backdrop({
	    "test": {
		"appstore":	APPSTORE_DNA_PATH,
	    },
	}, {
	    "app_port": APP_PORT,
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

	client				= new AppInterfaceClient( APP_PORT, {
	    "logging": process.env.LOG_LEVEL || "normal",
	});
	app_client			= await client.app( "test-alice" );

	alice_mm			= app_client.createZomeInterface(
	    "appstore",
	    "mere_memory_api",
	    MereMemoryZomelet,
	).functions;
    });

    describe("Publisher", publisher_tests.bind( this, holochain ) );
    describe("App", app_tests.bind( this, holochain ) );
    describe("Errors", errors_tests.bind( this, holochain ) );

    after(async () => {
	await holochain.destroy();
    });

});


let publisher_1;

function publisher_tests () {

    it("should create publisher profile", async function () {
	this.timeout( 10_000 );

	const publisher = publisher_1	= await clients.alice.call("appstore", "appstore_api", "create_publisher", await createPublisherInput( alice_mm ) );

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
	const publisher		= await clients.alice.call("appstore", "appstore_api", "create_publisher", await createPublisherInput( alice_mm ) );

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


let app_1;

function app_tests () {

    it("should create app profile", async function () {
	this.timeout( 10_000 );

	const input			= await createAppInput( alice_mm, {
	    "publisher": publisher_1.$id,
	});
	const app = app_1		= await clients.alice.call("appstore", "appstore_api", "create_app", input );

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
	const input		= await createAppInput( alice_mm, {
	    "publisher": publisher_1.$id,
	});
	const app		= await clients.alice.call("appstore", "appstore_api", "create_app", input );

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
	}, "InvalidCommit error: Previous entry author does not match Action author" );
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
	}, "InvalidCommit error: Previous entry author does not match Action author" );
    });

    it("should fail to create publisher because icon is too big", async function () {
	this.timeout( 10_000 );

	await expect_reject( async () => {
	    const input			= await createPublisherInput( alice_mm, {
		"icon": new Uint8Array( ICON_SIZE_LIMIT + 1 ).fill(0),
	    });
	    await clients.alice.call("appstore", "appstore_api", "create_publisher", input );
	}, `InvalidCommit error: PublisherEntry icon cannot be larger than ${Math.floor(ICON_SIZE_LIMIT/1024)}KB (${ICON_SIZE_LIMIT} bytes)` );
    });

    it("should fail to update publisher because icon is too big", async function () {
	this.timeout( 10_000 );

	await expect_reject( async () => {
	    const bytes			= new Uint8Array( ICON_SIZE_LIMIT + 1 ).fill(0);
	    const memory_addr		= await alice_mm.save( bytes );

	    await clients.alice.call("appstore", "appstore_api", "update_publisher", {
		"base": publisher_1.$action,
		"properties": {
		    "icon": memory_addr,
		},
	    });
	}, `InvalidCommit error: PublisherEntry icon cannot be larger than ${Math.floor(ICON_SIZE_LIMIT/1024)}KB (${ICON_SIZE_LIMIT} bytes)` );
    });

    it("should fail to create app because icon is too big", async function () {
	this.timeout( 10_000 );

	await expect_reject( async () => {
	    const input			= await createAppInput( alice_mm, {
		"icon": new Uint8Array( ICON_SIZE_LIMIT + 1 ).fill(0),
		"publisher": publisher_1.$id,
	    });
	    await clients.alice.call("appstore", "appstore_api", "create_app", input );
	}, `InvalidCommit error: AppEntry icon cannot be larger than ${Math.floor(ICON_SIZE_LIMIT/1024)}KB (${ICON_SIZE_LIMIT} bytes)` );
    });

    it("should fail to update app because icon is too big", async function () {
	this.timeout( 10_000 );

	await expect_reject( async () => {
	    const bytes			= new Uint8Array( ICON_SIZE_LIMIT + 1 ).fill(0);
	    const memory_addr		= await alice_mm.save( bytes );

	    await clients.alice.call("appstore", "appstore_api", "update_app", {
		"base": app_1.$action,
		"properties": {
		    "icon": memory_addr,
		},
	    });
	}, `InvalidCommit error: AppEntry icon cannot be larger than ${Math.floor(ICON_SIZE_LIMIT/1024)}KB (${ICON_SIZE_LIMIT} bytes)` );
    });
}
