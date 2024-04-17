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

import { Holochain }			from '@spartan-hc/holochain-backdrop';

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
    createPublisherInput,
    createGroupInput,
}					from '../utils.js';


const __dirname				= path.dirname( new URL(import.meta.url).pathname );
const APPSTORE_DNA_PATH			= path.join( __dirname, "../../dnas/appstore.dna" );

let app_port;
let client;
let app_client
let bobby_client;

let appstore_csr;
let bobby_appstore_csr;


describe("Controlled Viewpoint", () => {
    const holochain			= new Holochain({
	"timeout": 60_000,
	"default_stdout_loggers": log.level_rank > 3,
    });

    before(async function () {
	this.timeout( 60_000 );

	await holochain.install([
	    "alice",
	    "bobby",
	], [
	    {
		"app_name": "test",
		"bundle": {
		    "appstore":	APPSTORE_DNA_PATH,
		},
	    },
	]);

	app_port			= await holochain.ensureAppPort();

	client				= new AppInterfaceClient( app_port, {
	    "logging": process.env.LOG_LEVEL || "normal",
	});
	app_client			= await client.app( "test-alice" );
	bobby_client			= await client.app( "test-bobby" );

	{
	    const {
		appstore,
	    }				= app_client.createInterface({
		"appstore":	AppStoreCell,
	    });

	    appstore_csr		= appstore.zomes.appstore_csr.functions;
	}

	{
	    const bobby_appstore	= bobby_client.createCellInterface( "appstore", AppStoreCell );

	    bobby_appstore_csr		= bobby_appstore.zomes.appstore_csr.functions;
	}

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


let publisher1;

function publisher_tests () {

    it("should create publisher profile", async function () {
	this.timeout( 10_000 );

	const publisher = publisher1	= await appstore_csr.create_publisher(
	    createPublisherInput()
	);

	// log.debug( json.debug( publisher ) );

	expect( publisher.editors	).to.have.length( 2 );
    });

}


let app1;

function app_tests () {

    it("should create app profile", async function () {
	this.timeout( 10_000 );

	const input			= createAppInput({
	    "publisher": publisher1.$id,
	});
	const app = app1		= await appstore_csr.create_app( input );

	log.normal("App ID: %s", json.debug( app.$id ) );

	expect( app.editors		).to.have.length( 2 );
    });

}


let group1;

function group_tests () {

    it("should create group viewpoint", async function () {
	this.timeout( 10_000 );

	const group_input		= createGroupInput(
	    [
		app_client.agent_id,
	    ],
	);
	group1				= await appstore_csr.create_group( group_input );

	log.debug("Group: %s", json.debug( group1 ) );

	expect( group1.admins		).to.have.length( 1 );
    });

    it("should create group viewpoint", async function () {
	{
	    const apps			= await appstore_csr.get_all_apps();

	    expect( apps		).to.have.length( 1 );
	}
	const apps			= await group1.$getAllApps();

	log.debug( json.debug( apps ) );

	expect( apps			).to.have.length( 1 );

	const ma_state			= await group1.$getAppModeratedState( app1.$id );

	expect( ma_state		).to.be.null;
    });

    it("should remove app from group view", async function () {
	const moderator_action		= await group1.$removeApp(
	    app1.$id,
	    "App fails to install and developer cannot be contacted"
	);

	log.debug("Removed app: %s", json.debug(moderator_action) );

	{
	    const apps			= await group1.$getAllApps();
	    log.debug( json.debug( apps ) );
	    expect( apps		).to.have.length( 0 );
	}
	{
	    const apps			= await group1.$getAllRemovedApps();
	    log.debug( json.debug( apps ) );
	    expect( apps		).to.have.length( 1 );
	}

	const ma_state			= await group1.$getAppModeratedState( app1.$id );

	expect( ma_state.message	).to.equal( moderator_action.message );
    });

    it("should unremove app from group view", async function () {
	const updated_ma_entry			= await group1.$unremoveApp(
	    app1.$id,
	    "Developer fixed the app"
	);

	log.debug("Unremoved app: %s", json.debug(updated_ma_entry) );

	const apps			= await group1.$getAllApps();

	log.debug( json.debug( apps ) );

	expect( apps			).to.have.length( 1 );

	const ma_state			= await group1.$getAppModeratedState( app1.$id );

	expect( ma_state.message	).to.equal( updated_ma_entry.message );
    });

    it("should get moderator actions", async function () {
	const moderator_actions		= await group1.$getAppModeratedActions( app1.$id );

	log.debug( json.debug( moderator_actions ) );
    });

}


const ICON_SIZE_LIMIT		= 204_800;

function errors_tests () {

    it("should fail to remove app because agent is not a group member", async function () {
	await expect_reject( async () => {
	    const moderator_action		= await bobby_appstore_csr.update_moderated_state({
		"group_id": group1.$id,
		"app_id": app1.$id,
		"message": "malicious",
		"metadata": {
		    "remove": true,
		},
	    });
	}, "is not authorized to update content managed by group" );
    });

}
