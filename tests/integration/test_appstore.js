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
}					from '@holochain/appstore-zomelets';
import {
    AppInterfaceClient,
}					from '@spartan-hc/app-interface-client';

import {
    expect_reject,
    linearSuite,
    createAppInput,
    createAppVersionInput,
    createPublisherInput,
}					from '../utils.js';


const __dirname				= path.dirname( new URL(import.meta.url).pathname );
const APPSTORE_DNA_PATH			= path.join( __dirname, "../../dnas/appstore.dna" );

let app_port;
let client;
let alice_client;
let bobby_client;
let carol_client;

let alice_appstore;
let bobby_appstore;
let carol_appstore;


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
	    "actors": [
		"alice", // publisher1 - admin
		"bobby", // publisher1 - member
		"carol", // publisher2 - admin
		// "david",
		// "emily",
		// "felix",
	    ],
	});

	app_port			= await holochain.appPorts()[0];

	client				= new AppInterfaceClient( app_port, {
	    "logging": process.env.LOG_LEVEL || "normal",
	});
	alice_client			= await client.app( "test-alice" );
	bobby_client			= await client.app( "test-bobby" );
	carol_client			= await client.app( "test-carol" );

	{
	    const {
		appstore,
	    }				= alice_client.createInterface({
		"appstore":	AppStoreCell,
	    });

	    alice_appstore		= appstore.zomes.appstore_csr.functions;
	}

	{
	    const {
		appstore,
	    }				= bobby_client.createInterface({
		"appstore":	AppStoreCell,
	    });

	    bobby_appstore		= appstore.zomes.appstore_csr.functions;
	}

	{
	    const {
		appstore,
	    }				= carol_client.createInterface({
		"appstore":	AppStoreCell,
	    });

	    carol_appstore		= appstore.zomes.appstore_csr.functions;
	}

	// Must call whoami on each cell to ensure that init has finished.
	await alice_appstore.whoami();
	await bobby_appstore.whoami();
	await carol_appstore.whoami();
    });

    linearSuite("Publisher", publisher_tests.bind( this, holochain ) );
    linearSuite("App", app_tests.bind( this, holochain ) );
    linearSuite("App Version", app_version_tests.bind( this, holochain ) );
    linearSuite("Errors", errors_tests.bind( this, holochain ) );

    after(async () => {
	await holochain.destroy();
    });

});


let publisher1;

function publisher_tests () {

    it("should create publisher profile", async function () {
	this.timeout( 10_000 );

	publisher1			= await alice_appstore.create_publisher(
	    createPublisherInput({
		"editors": [
		    bobby_client.agent_id,
		],
	    })
	);

	// log.debug( json.debug( publisher ) );
    });

    it("should get publisher profile", async function () {
	const publisher = publisher1	= await alice_appstore.get_publisher( publisher1.$id );

	expect( publisher.$id		).to.deep.equal( publisher1.$id );
    });

    it("should get publishers for a group", async function () {
	{
	    const publishers		= await alice_appstore.get_publishers_for_group({
		"for_group": publisher1.editors_group_id[0],
	    });

	    expect( publishers		).to.have.length( 1 );
	}
    });

    it("should get publishers for an agent", async function () {
	{
	    const publishers		= await alice_appstore.get_publishers_for_agent({
		"for_agent": alice_client.agent_id,
	    });

	    expect( publishers		).to.have.length( 1 );
	}
	{
	    const publishers		= await alice_appstore.get_publishers_for_agent({
		"for_agent": bobby_client.agent_id,
	    });

	    expect( publishers		).to.have.length( 1 );
	}
    });

    it("should get my publishers", async function () {
	const publishers		= await alice_appstore.get_my_publishers();

	expect( publishers		).to.have.length( 1 );
    });

    it("should update publisher profile", async function () {
	const new_name			= "Holo Inc";

	await bobby_appstore.update_publisher({
	    "base": publisher1.$action,
	    "properties": {
		"name": new_name,
	    },
	});

	await publisher1.$refresh();

	expect( publisher1.name		).to.equal( new_name );
    });

    it("should deprecate publisher", async function () {
	const publisher			= await alice_appstore.create_publisher(
	    createPublisherInput()
	);

	{
	    const publishers	= await alice_appstore.get_my_publishers();
	    expect( publishers	).to.have.length( 2 );
	}

	await publisher1.$deprecate( "Oopsie!" );

	{
	    const publishers	= await alice_appstore.get_my_publishers();
	    expect( publishers	).to.have.length( 2 );
	}

	await publisher1.$undeprecate();
    });

    it("should get editors groups for agent", async function () {
	{
	    let groups		= await alice_appstore.get_editors_groups_for_agent({
		"for_agent": alice_client.agent_id,
	    });

	    expect( groups	).to.have.length( 2 );
	}
    });

}


let app1;

function app_tests () {

    it("should create app profile", async function () {
	this.timeout( 10_000 );

	const input			= createAppInput({
	    "publisher": publisher1.$id,
	});
	app1				= await alice_appstore.create_app( input );

	// log.debug( json.debug( app ) );
    });

    it("should get app profile", async function () {
	const app			= await alice_appstore.get_app( app1.$id );

	expect( app.$id			).to.deep.equal( app1.$id );
    });

    it("should get apps for an agent", async function () {
	const apps			= await alice_appstore.get_apps_for_agent({
	    "for_agent": alice_client.agent_id,
	});

	expect( apps		).to.have.length( 1 );
    });

    it("should get apps for publisher", async function () {
	const apps			= await alice_appstore.get_apps_for_publisher({
	    "for_publisher": publisher1.$id,
	});

	expect( apps		).to.have.length( 1 );
    });

    it("should get my apps", async function () {
	const apps		= await alice_appstore.get_my_apps();

	expect( apps		).to.have.length( 1 );
    });

    it("should get all apps", async function () {
	const apps		= await alice_appstore.get_all_apps();

	expect( apps		).to.have.length( 1 );
    });

    it("should deprecate app", async function () {
	const input		= createAppInput({
	    "publisher": publisher1.$id,
	});
	const app		= await alice_appstore.create_app( input );

	{
	    const apps		= await alice_appstore.get_my_apps();
	    expect( apps	).to.have.length( 2 );
	}
	{
	    const apps		= await alice_appstore.get_all_apps();
	    expect( apps	).to.have.length( 2 );
	}

	await app1.$deprecate( "Oopsie!" );

	{
	    const apps		= await alice_appstore.get_my_apps();
	    expect( apps	).to.have.length( 2 );
	}
	{
	    const apps		= await alice_appstore.get_all_apps();
	    expect( apps	).to.have.length( 1 );
	}

	await app1.$undeprecate();
    });

}


let app_version1;

function app_version_tests () {

    it("should create app version", async function () {
	this.timeout( 10_000 );

	const input			= createAppVersionInput({
	    "version": "0.1.0",
	    "for_app": app1.$id,
	    "bundle_hashes": {
		"hash": "",
		"ui_hash": "",
		"happ_hash": "",
	    },
	});
	app_version1			= await alice_appstore.create_app_version( input );

	log.normal("%s", json.debug( app_version1 ) );
    });

    it("should get versions for app", async function () {
	this.timeout( 10_000 );

	const versions			= await app1.$getVersions();

	expect( versions		).to.have.length( 1 );
    });

}


const ICON_SIZE_LIMIT		= 204_800;

function errors_tests () {
    it("should fail to update publisher because bad author", async function () {
	this.timeout( 10_000 );

	await expect_reject( async () => {
	    await carol_appstore.update_publisher({
		"base": publisher1.$action,
		"properties": {
		    "name": "Malicious",
		},
	    });
	}, "Invalid editor" );
    });

    it("should fail to update app because bad author", async function () {
	this.timeout( 10_000 );

	await expect_reject( async () => {
	    await carol_appstore.update_app({
		"base": app1.$action,
		"properties": {
		    "name": "Malicious",
		},
	    });
	}, "Invalid editor" );
    });

    it("should fail to create publisher because icon is too big", async function () {
	this.timeout( 10_000 );

	await expect_reject( async () => {
	    const input			= createPublisherInput({
		"icon": new Uint8Array( ICON_SIZE_LIMIT + 1 ).fill(0),
	    });
	    await alice_appstore.create_publisher( input );
	}, `InvalidCommit error: PublisherEntry icon cannot be larger than ${Math.floor(ICON_SIZE_LIMIT/1024)}KB (${ICON_SIZE_LIMIT} bytes)` );
    });

    it("should fail to update publisher because icon is too big", async function () {
	this.timeout( 10_000 );

	await expect_reject( async () => {
	    await publisher1.$update({
		"icon": new Uint8Array( ICON_SIZE_LIMIT + 1 ).fill(0),
	    });
	}, `InvalidCommit error: PublisherEntry icon cannot be larger than ${Math.floor(ICON_SIZE_LIMIT/1024)}KB (${ICON_SIZE_LIMIT} bytes)` );
    });

    it("should fail to create app because icon is too big", async function () {
	this.timeout( 10_000 );

	await expect_reject( async () => {
	    const input			= createAppInput({
		"icon": new Uint8Array( ICON_SIZE_LIMIT + 1 ).fill(0),
		"publisher": publisher1.$id,
	    });
	    await alice_appstore.create_app( input );
	}, `InvalidCommit error: AppEntry icon cannot be larger than ${Math.floor(ICON_SIZE_LIMIT/1024)}KB (${ICON_SIZE_LIMIT} bytes)` );
    });

    it("should fail to update app because icon is too big", async function () {
	this.timeout( 10_000 );

	await expect_reject( async () => {
	    await app1.$update({
		"icon": new Uint8Array( ICON_SIZE_LIMIT + 1 ).fill(0),
	    });
	}, `InvalidCommit error: AppEntry icon cannot be larger than ${Math.floor(ICON_SIZE_LIMIT/1024)}KB (${ICON_SIZE_LIMIT} bytes)` );
    });

    it("should fail to create app version because invalid author", async function () {
	this.timeout( 10_000 );

	await expect_reject( async () => {
	    const input			= createAppVersionInput({
		"version": "0.1.0",
		"for_app": app1.$id,
		"bundle_hashes": {
		    "hash": "",
		    "ui_hash": "",
		    "happ_hash": "",
		},
	    });
	    await carol_appstore.create_app_version( input );
	}, "not authorized" );
    });
}
