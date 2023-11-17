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

import HolochainBackdrop		from '@spartan-hc/holochain-backdrop';
const { Holochain }			= HolochainBackdrop;

import {
    AppStoreCell,
    MereMemoryZomelet,
}					from '@holochain/appstore-zomelets';
import {
    AppInterfaceClient,
}					from '@spartan-hc/app-interface-client';

import {
    expect_reject,
    linearSuite,
    createAppInput,
    createPublisherInput,
}					from '../utils.js';


const delay				= (n) => new Promise(f => setTimeout(f, n));

const __dirname				= path.dirname( new URL(import.meta.url).pathname );
const APPSTORE_DNA_PATH			= path.join( __dirname, "../../dnas/appstore.dna" );
const TEST_DNA_HASH			= "uhC0kXracwD-PyrSU5m_unW3GA7vV1fY1eHH-0qV5HG7Y7s-DwLa5";
const APP_PORT				= 23_567;

let client;
let app_client, bobby_client;
let appstore, appstore_csr;
let bobby_appstore, bobby_appstore_csr;


describe("Appstore", () => {
    const holochain			= new Holochain({
	"timeout": 60_000,
	"default_stdout_loggers": log.level_rank > 3,
    });

    before(async function () {
	this.timeout( 60_000 );

	await holochain.backdrop({
	    "test": {
		"appstore":	APPSTORE_DNA_PATH,
	    },
	}, {
	    "app_port": APP_PORT,
	    "actors": [ "alice", "bobby" ],
	});

	client				= new AppInterfaceClient( APP_PORT, {
	    "logging": process.env.LOG_LEVEL || "normal",
	});
	app_client			= await client.app( "test-alice" );
	bobby_client			= await client.app( "test-bobby" );

	({
	    appstore,
	}				= app_client.createInterface({
	    "appstore":	AppStoreCell,
	}));

	bobby_appstore			= bobby_client.createCellInterface( "appstore", AppStoreCell );

	appstore_csr			= appstore.zomes.appstore_csr.functions;
	bobby_appstore_csr		= bobby_appstore.zomes.appstore_csr.functions;

	// Must call whoami on each cell to ensure that init has finished.
	await appstore_csr.whoami();
	await bobby_appstore_csr.whoami();
    });

    linearSuite("Publisher", publisher_tests.bind( this, holochain ) );
    linearSuite("App", app_tests.bind( this, holochain ) );
    linearSuite("Errors", errors_tests.bind( this, holochain ) );

    after(async () => {
	await holochain.destroy();
    });

});


let publisher_1;

function publisher_tests () {

    it("should create publisher profile", async function () {
	this.timeout( 10_000 );

	const publisher = publisher_1	= await appstore_csr.create_publisher(
	    await createPublisherInput()
	);

	// log.debug( json.debug( publisher ) );

	expect( publisher.editors	).to.have.length( 2 );
    });

    it("should get publisher profile", async function () {
	const publisher = publisher_1	= await appstore_csr.get_publisher( publisher_1.$id );

	expect( publisher.$id		).to.deep.equal( publisher_1.$id );
    });

    it("should get publishers for an agent", async function () {
	const publishers		= await appstore_csr.get_publishers_for_agent({
	    "for_agent": app_client.agent_id,
	});

	expect( publishers		).to.have.length( 1 );
    });

    it("should get my publishers", async function () {
	const publishers		= await appstore_csr.get_my_publishers();

	expect( publishers		).to.have.length( 1 );
    });

    it("should update publisher profile", async function () {
	const publisher = publisher_1	= await appstore_csr.update_publisher({
	    "base": publisher_1.$action,
	    "properties": {
		"name": "Holo Inc",
	    },
	});

	expect( publisher.name		).to.equal( "Holo Inc" );
    });

    it("should get all publishers", async function () {
	const publishers		= await appstore_csr.get_all_publishers();

	expect( publishers		).to.have.length( 1 );
    });

    it("should deprecate publisher", async function () {
	const publisher			= await appstore_csr.create_publisher(
	    await createPublisherInput()
	);

	{
	    const publishers	= await appstore_csr.get_my_publishers();
	    expect( publishers	).to.have.length( 2 );
	}
	{
	    const publishers	= await appstore_csr.get_all_publishers();
	    expect( publishers	).to.have.length( 2 );
	}

	await appstore_csr.deprecate_publisher({
	    "base": publisher.$action,
	    "message": "Oopsie!",
	});

	{
	    const publishers	= await appstore_csr.get_my_publishers();
	    expect( publishers	).to.have.length( 2 );
	}
	{
	    const publishers	= await appstore_csr.get_all_publishers();
	    expect( publishers	).to.have.length( 1 );
	}
    });

}


let app_1;

function app_tests () {

    it("should create app profile", async function () {
	this.timeout( 10_000 );

	const input			= await createAppInput({
	    "publisher": publisher_1.$id,
	});
	const app = app_1		= await appstore_csr.create_app( input );

	// log.debug( json.debug( app ) );

	expect( app.editors		).to.have.length( 2 );
    });

    it("should get app profile", async function () {
	const app			= await appstore_csr.get_app( app_1.$id );

	expect( app.$id			).to.deep.equal( app_1.$id );
    });

    it("should get apps for an agent", async function () {
	const apps			= await appstore_csr.get_apps_for_agent({
	    "for_agent": app_client.agent_id,
	});

	expect( apps		).to.have.length( 1 );
    });

    it("should get my apps", async function () {
	const apps		= await appstore_csr.get_my_apps();

	expect( apps		).to.have.length( 1 );
    });

    it("should get all apps", async function () {
	const apps		= await appstore_csr.get_all_apps();

	expect( apps		).to.have.length( 1 );
    });

    it("should deprecate app", async function () {
	const input		= await createAppInput({
	    "publisher": publisher_1.$id,
	});
	const app		= await appstore_csr.create_app( input );

	{
	    const apps		= await appstore_csr.get_my_apps();
	    expect( apps	).to.have.length( 2 );
	}
	{
	    const apps		= await appstore_csr.get_all_apps();
	    expect( apps	).to.have.length( 2 );
	}

	await appstore_csr.deprecate_app({
	    "base": app.$action,
	    "message": "Oopsie!",
	});

	{
	    const apps		= await appstore_csr.get_my_apps();
	    expect( apps	).to.have.length( 2 );
	}
	{
	    const apps		= await appstore_csr.get_all_apps();
	    expect( apps	).to.have.length( 1 );
	}
    });

}


const ICON_SIZE_LIMIT		= 204_800;

function errors_tests () {
    it("should fail to update publisher because bad author", async function () {
	this.timeout( 10_000 );

	await expect_reject( async () => {
	    await bobby_appstore_csr.update_publisher({
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
	    await bobby_appstore_csr.update_app({
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
	    const input			= await createPublisherInput({
		"icon": new Uint8Array( ICON_SIZE_LIMIT + 1 ).fill(0),
	    });
	    await appstore_csr.create_publisher( input );
	}, `InvalidCommit error: PublisherEntry icon cannot be larger than ${Math.floor(ICON_SIZE_LIMIT/1024)}KB (${ICON_SIZE_LIMIT} bytes)` );
    });

    it("should fail to update publisher because icon is too big", async function () {
	this.timeout( 10_000 );

	await expect_reject( async () => {
	    await appstore_csr.update_publisher({
		"base": publisher_1.$action,
		"properties": {
		    "icon": new Uint8Array( ICON_SIZE_LIMIT + 1 ).fill(0),
		},
	    });
	}, `InvalidCommit error: PublisherEntry icon cannot be larger than ${Math.floor(ICON_SIZE_LIMIT/1024)}KB (${ICON_SIZE_LIMIT} bytes)` );
    });

    it("should fail to create app because icon is too big", async function () {
	this.timeout( 10_000 );

	await expect_reject( async () => {
	    const input			= await createAppInput({
		"icon": new Uint8Array( ICON_SIZE_LIMIT + 1 ).fill(0),
		"publisher": publisher_1.$id,
	    });
	    await appstore_csr.create_app( input );
	}, `InvalidCommit error: AppEntry icon cannot be larger than ${Math.floor(ICON_SIZE_LIMIT/1024)}KB (${ICON_SIZE_LIMIT} bytes)` );
    });

    it("should fail to update app because icon is too big", async function () {
	this.timeout( 10_000 );

	await expect_reject( async () => {
	    await appstore_csr.update_app({
		"base": app_1.$action,
		"properties": {
		    "icon": new Uint8Array( ICON_SIZE_LIMIT + 1 ).fill(0),
		},
	    });
	}, `InvalidCommit error: AppEntry icon cannot be larger than ${Math.floor(ICON_SIZE_LIMIT/1024)}KB (${ICON_SIZE_LIMIT} bytes)` );
    });
}
