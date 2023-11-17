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
    createGroupInput,
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


describe("Controlled Viewpoint", () => {
    const holochain			= new Holochain({
	"timeout": 60_000,
	"default_stdout_loggers": process.env.LOG_LEVEL === "silly",
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
    linearSuite("Group Viewpoint", group_tests.bind( this, holochain ) );
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

}


let app_1;

function app_tests () {

    it("should create app profile", async function () {
	this.timeout( 10_000 );

	const input			= await createAppInput({
	    "publisher": publisher_1.$id,
	});
	const app = app_1		= await appstore_csr.create_app( input );

	log.normal("App ID: %s", json.debug( app.$id ) );

	expect( app.editors		).to.have.length( 2 );
    });

}


let group_1;

function group_tests () {

    it("should create group viewpoint", async function () {
	this.timeout( 10_000 );

	const group_input		= await createGroupInput(
	    [
		app_client.agent_id,
	    ],
	);
	group_1				= await appstore_csr.create_group( group_input );

	log.debug("Group: %s", json.debug( group_1 ) );

	// expect( publisher.editors	).to.have.length( 2 );
    });

    it("should create group viewpoint", async function () {
	{
	    const apps			= await appstore_csr.get_all_apps();

	    expect( apps		).to.have.length( 1 );
	}
	const apps			= await appstore_csr.viewpoint_get_all_apps( group_1.$id );

	log.debug( json.debug( apps ) );

	expect( apps			).to.have.length( 1 );

	const ma_state			= await appstore_csr.get_moderated_state({
	    "group_id": group_1.$id,
	    "app_id": app_1.$id,
	});

	expect( ma_state		).to.be.null;
    });

    it("should remove app from group view", async function () {
	const moderator_action		= await appstore_csr.update_moderated_state({
	    "group_id": group_1.$id,
	    "app_id": app_1.$id,
	    "message": "App fails to install and developer cannot be contacted",
	    "metadata": {
		"remove": true,
	    },
	});

	log.debug("Removed app: %s", json.debug(moderator_action) );

	{
	    const apps			= await appstore_csr.viewpoint_get_all_apps( group_1.$id );
	    log.debug( json.debug( apps ) );
	    expect( apps		).to.have.length( 0 );
	}
	{
	    const apps			= await appstore_csr.viewpoint_get_all_removed_apps( group_1.$id );
	    log.debug( json.debug( apps ) );
	    expect( apps		).to.have.length( 1 );
	}

	const ma_state			= await appstore_csr.get_moderated_state({
	    "group_id": group_1.$id,
	    "app_id": app_1.$id,
	});

	expect( ma_state.message	).to.equal( moderator_action.message );
    });

    it("should unremove app from group view", async function () {
	const updated_ma_entry			= await appstore_csr.update_moderated_state({
	    "group_id": group_1.$id,
	    "app_id": app_1.$id,
	    "message": "Developer fixed the app",
	    "metadata": {
		"remove": false,
	    },
	});

	log.debug("Unremoved app: %s", json.debug(updated_ma_entry) );

	const apps			= await appstore_csr.viewpoint_get_all_apps( group_1.$id );

	log.debug( json.debug( apps ) );

	expect( apps			).to.have.length( 1 );

	const ma_state			= await appstore_csr.get_moderated_state({
	    "group_id": group_1.$id,
	    "app_id": app_1.$id,
	});

	expect( ma_state.message	).to.equal( updated_ma_entry.message );
    });

    it("should get moderator actions", async function () {
	const moderator_actions		= await appstore_csr.get_moderator_actions({
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
	    const moderator_action		= await bobby_appstore_csr.update_moderated_state({
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
