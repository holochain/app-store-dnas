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


describe("Controlled Viewpoint", () => {
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
    describe("Group Viewpoint", group_tests.bind( this, holochain ) );
    describe("Errors", errors_tests.bind( this, holochain ) );

    after(async () => {
	await holochain.destroy();
    });

});


let publisher_1;

function publisher_tests () {

    it("should create publisher profile", async function () {
	this.timeout( 10_000 );

	const publisher = publisher_1	= await clients.alice.call("appstore", "appstore_api", "create_publisher", createPublisherInput( alice_mm ) );

	// log.debug( json.debug( publisher ) );

	expect( publisher.editors	).to.have.length( 2 );
    });

}


let app_1;

function app_tests () {

    it("should create app profile", async function () {
	this.timeout( 10_000 );

	const input			= createAppInput( alice_mm, {
	    "publisher": publisher_1.$id,
	});
	const app = app_1		= await clients.alice.call("appstore", "appstore_api", "create_app", input );

	log.normal("App ID: %s", json.debug( app.$id ) );

	expect( app.editors		).to.have.length( 2 );
    });

}


let group_1;

function createGroupInput ( admins, ...members ) {
    return {
	"admins": admins,
	"members": [ ...members ],

	"published_at":		Date.now(),
	"last_updated":		Date.now(),
	"metadata":		{},
    };
};

function group_tests () {

    it("should create group viewpoint", async function () {
	this.timeout( 10_000 );

	const group_input		= createGroupInput(
	    [
		clients.alice.cellAgent(),
	    ],
	);
	group_1				= await clients.alice.call("appstore", "appstore_api", "create_group", group_input );

	console.log( group_1 );
	log.debug("Group: %s", json.debug( group_1 ) );

	// expect( publisher.editors	).to.have.length( 2 );
    });

    it("should create group viewpoint", async function () {
	{
	    const apps			= await clients.alice.call("appstore", "appstore_api", "get_all_apps" );

	    expect( apps		).to.have.length( 1 );
	}
	const apps			= await clients.alice.call("appstore", "appstore_api", "viewpoint_get_all_apps", group_1.$id );

	log.debug( json.debug( apps ) );

	expect( apps			).to.have.length( 1 );

	const ma_state			= await clients.alice.call("appstore", "appstore_api", "get_moderated_state", {
	    "group_id": group_1.$id,
	    "app_id": app_1.$id,
	});

	expect( ma_state		).to.be.null;
    });

    it("should remove app from group view", async function () {
	const moderator_action		= await clients.alice.call("appstore", "appstore_api", "update_moderated_state", {
	    "group_id": group_1.$id,
	    "app_id": app_1.$id,
	    "message": "App fails to install and developer cannot be contacted",
	    "metadata": {
		"remove": true,
	    },
	});

	log.debug("Removed app: %s", json.debug(moderator_action) );

	{
	    const apps			= await clients.alice.call("appstore", "appstore_api", "viewpoint_get_all_apps", group_1.$id );
	    log.debug( json.debug( apps ) );
	    expect( apps		).to.have.length( 0 );
	}
	{
	    const apps			= await clients.alice.call("appstore", "appstore_api", "viewpoint_get_all_removed_apps", group_1.$id );
	    log.debug( json.debug( apps ) );
	    expect( apps		).to.have.length( 1 );
	}

	const ma_state			= await clients.alice.call("appstore", "appstore_api", "get_moderated_state", {
	    "group_id": group_1.$id,
	    "app_id": app_1.$id,
	});

	expect( ma_state.message	).to.equal( moderator_action.message );
    });

    it("should unremove app from group view", async function () {
	const updated_ma_entry			= await clients.alice.call("appstore", "appstore_api", "update_moderated_state", {
	    "group_id": group_1.$id,
	    "app_id": app_1.$id,
	    "message": "Developer fixed the app",
	    "metadata": {
		"remove": false,
	    },
	});

	log.debug("Unremoved app: %s", json.debug(updated_ma_entry) );

	const apps			= await clients.alice.call("appstore", "appstore_api", "viewpoint_get_all_apps", group_1.$id );

	log.debug( json.debug( apps ) );

	expect( apps			).to.have.length( 1 );

	const ma_state			= await clients.alice.call("appstore", "appstore_api", "get_moderated_state", {
	    "group_id": group_1.$id,
	    "app_id": app_1.$id,
	});

	expect( ma_state.message	).to.equal( updated_ma_entry.message );
    });

    it("should get moderator actions", async function () {
	const moderator_actions		= await clients.alice.call("appstore", "appstore_api", "get_moderator_actions", {
	    "group_id": group_1.$id,
	    "app_id": app_1.$id,
	});

	log.debug( json.debug( moderator_actions ) );
    });

}


const ICON_SIZE_LIMIT		= 204_800;

function errors_tests () {

    it("should fail to remove app because agent is not a group member", async function () {
	await expect_reject( async () => {
	    const moderator_action		= await clients.bobby.call("appstore", "appstore_api", "update_moderated_state", {
		"group_id": group_1.$id,
		"app_id": app_1.$id,
		"message": "malicious",
		"metadata": {
		    "remove": true,
		},
	    });
	}, "is not authorized to update content managed by group" );
    });

}
