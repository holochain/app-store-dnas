import {
    AnyDhtHash,
    AgentPubKey,
    ActionHash, EntryHash,
}					from '@spartan-hc/holo-hash'; // approx. 11kb
import {
    Zomelet,
    CellZomelets,
}					from '@spartan-hc/zomelets'; // approx. 7kb
import { MereMemoryZomelet }		from '@spartan-hc/mere-memory-zomelets'; // approx. 33kb
import {
    PortalCell,
}					from '@holochain/portal-zomelets';
import {
    AppHubCell,
    DnaHubCell,
    ZomeHubCell,
    Bundle,
    semverReverseSort,
}					from '@holochain/apphub-zomelets';
import {
    Publisher,
    App,
    AppVersion,
    Group,
}					from './types.js';


async function hash_entry ( input ) {
    const result			= await this.call( input );

    return new EntryHash( result );
}


export const AppStoreCSRZomelet		= new Zomelet({
    "whoami": {
	output ( response ) {
	    // Struct - https://docs.rs/hdk/*/hdk/prelude/struct.AgentInfo.html
	    return {
		"pubkey": {
		    "initial":		new AgentPubKey( response.agent_initial_pubkey ),
		    "latest":		new AgentPubKey( response.agent_latest_pubkey ),
		},
		"chain_head": {
		    "action":		new ActionHash( response.chain_head[0] ),
		    "sequence":		response.chain_head[1],
		    "timestamp":	response.chain_head[2],
		},
	    };
	},
    },

    //
    // Publisher
    //
    async create_publisher_entry ( input ) {
	const result			= await this.call( input );

	return new Publisher( result, this );
    },
    async create_publisher ( input ) {
	if ( input.icon && input.icon.length > 39 )
	    input.icon			= await this.zomes.mere_memory_api.save( input.icon );

	const result			= await this.call( input );

	return new Publisher( result, this );
    },
    async get_publisher_entry ( input ) {
	const result			= await this.call( new AnyDhtHash( input ) );

	return new Publisher( result, this );
    },
    async get_publisher ( input ) {
	const result			= await this.call({
	    "id": new ActionHash( input ),
	});

	return new Publisher( result, this );
    },
    "get_publishers_for_agent":		true,
    "get_my_publishers":		true,
    "get_all_publishers":		true,
    async update_publisher ( input ) {
	if ( input.properties.icon && input.properties.icon.length > 39 )
	    input.properties.icon	= await this.zomes.mere_memory_api.save( input.properties.icon );

	const result			= await this.call( input );

	return new Publisher( result, this );
    },
    async deprecate_publisher ( input ) {
	const result			= await this.call( input );

	return new Publisher( result, this );
    },
    async undeprecate_publisher ( input ) {
	const result			= await this.call( input );

	return new Publisher( result, this );
    },

    //
    // App
    //
    async create_app_entry ( input ) {
	const result			= await this.call( input );

	return new App( result, this );
    },
    async create_app ( input ) {
	if ( input.icon && input.icon.length > 39 )
	    input.icon			= await this.zomes.mere_memory_api.save( input.icon );

	const result			= await this.call( input );

	return new App( result, this );
    },
    async get_app_entry ( input ) {
	const result			= await this.call( new AnyDhtHash( input ) );

	return new App( result, this );
    },
    async get_app ( input ) {
	const result			= await this.call({
	    "id": new ActionHash( input ),
	});

	return new App( result, this );
    },
    async get_apps_for_publisher ( input ) {
	const result			= await this.call( input );

	return result.map( app => new App( app, this ) );
    },
    async get_apps_for_agent ( input ) {
	const result			= await this.call( input );

	return result.map( app => new App( app, this ) );
    },
    async get_my_apps ( input ) {
	const result			= await this.call( input );

	return result.map( app => new App( app, this ) );
    },
    async get_all_apps ( input ) {
	const result			= await this.call( input );

	return result.map( app => new App( app, this ) );
    },
    async update_app ( input ) {
	if ( input.properties.icon && input.properties.icon.length > 39 )
	    input.properties.icon	= await this.zomes.mere_memory_api.save( input.properties.icon );

	const result			= await this.call( input );

	return new App( result, this );
    },
    async deprecate_app ( input ) {
	const result			= await this.call( input );

	return new App( result, this );
    },
    async undeprecate_app ( input ) {
	const result			= await this.call( input );

	return new App( result, this );
    },

    //
    // App Version
    //
    async create_app_version ( input ) {
	if ( input.bundle_hashes === undefined ) {
	    input.bundle_hashes		= await this.functions.get_apphub_webapp_bundle_hashes({
		"dna":		input.apphub_hrl.dna,
		"target":	input.apphub_hrl.target,
		"hash":		input.apphub_hrl_hash,
	    });
	}

	const result			= await this.call( input );

	return new AppVersion( result, this );
    },
    async get_app_version ( input ) {
	const result			= await this.call({
	    "id": new ActionHash( input ),
	});

	return new AppVersion( result, this );
    },
    async get_app_versions_for_app ( input ) {
	const result			= await this.call( input );
	const version_map		= {};
	const versions			= [];

	for ( let app_version of result ) {
	    const vtag			= app_version.version;
	    version_map[ vtag ]		= new AppVersion( app_version, this );
	}

	semverReverseSort(
	    Object.keys( version_map )
	).forEach( vtag => {
	    versions.push( version_map[ vtag ] );
	});

	return versions;
    },

    //
    // Group
    //
    async create_group_entry ( input ) {
	const result			= await this.call( input );

	return new Group( result, this );
    },
    async create_group ( input ) {
	const result			= await this.call( input );

	return new Group( result, this );
    },
    async get_group_entry ( input ) {
	const result			= await this.call( new AnyDhtHash( input ) );

	return new Group( result, this );
    },
    async get_group ( input ) {
	const result			= await this.call({
	    "id": new ActionHash( input ),
	});

	return new Group( result, this );
    },
    "hash_ui_entry":				hash_entry,
    "hash_app_entry":				hash_entry,
    "hash_webapp_entry":			hash_entry,
    "hash_webapp_package_entry":		hash_entry,
    "hash_webapp_package_version_entry":	hash_entry,
    "hash_mere_memory_entry":			hash_entry,
    "hash_mere_memory_block_entry":		hash_entry,

    "get_moderator_actions":		true,
    "get_moderated_state":		true,
    "update_moderated_state":		true,
    "viewpoint_get_all_apps":		true,
    "viewpoint_get_all_removed_apps":	true,


    //
    // Virtual functions
    //
    async get_apphub_webapp_bundle_hashes ( input ) {
	const apphub			= this.getCellInterface( "apphub", input.dna );

	const webapp_version		= await this.functions.get_apphub_webapp_package_version(
	    input
	);
	const bundle_bytes		= await apphub.apphub_csr.get_webhapp_bundle(
	    webapp_version.webapp
	);
	const webapp_bundle		= new Bundle( bundle_bytes, "webhapp" );

	const calc_hash			= await this.zomes.mere_memory_api.calculate_hash(
	    bundle_bytes,
	);
	const calc_ui_hash		= await this.zomes.mere_memory_api.calculate_hash(
	    webapp_bundle.ui()
	);
	const calc_happ_hash		= await this.zomes.mere_memory_api.calculate_hash(
	    webapp_bundle.resources[ webapp_bundle.manifest.happ_manifest.bundled ]
	);

	return {
	    "hash":		calc_hash,
	    "ui_hash":		calc_ui_hash,
	    "happ_hash":	calc_happ_hash,
	};
    },
    async get_apphub_memory ( input ) {
	return await this.functions.get_apphub_entry({
	    "dna":			input.dna,
	    "zome":			"mere_memory_api",
	    "function":			"get_memory_entry",
	    "args":			input.target,
	    "hash_function_name":	"hash_mere_memory_entry",
	    "expected_hash":		input.hash || input.target,
	});
    },
    async get_apphub_memory_block ( input ) {
	return await this.functions.get_apphub_entry({
	    "dna":			input.dna,
	    "zome":			"mere_memory_api",
	    "function":			"get_memory_block_entry",
	    "args":			input.target,
	    "hash_function_name":	"hash_mere_memory_block_entry",
	    "expected_hash":		input.hash || input.target,
	});
    },
    async get_apphub_ui ( input ) {
	return await this.functions.get_apphub_entry({
	    "dna":			input.dna,
	    "zome":			"apphub_csr",
	    "function":			"get_ui_entry",
	    "args":			input.target,
	    "hash_function_name":	"hash_ui_entry",
	    "expected_hash":		input.hash || input.target,
	});
    },
    async get_apphub_app ( input ) {
	return await this.functions.get_apphub_entry({
	    "dna":			input.dna,
	    "zome":			"apphub_csr",
	    "function":			"get_app_entry",
	    "args":			input.target,
	    "hash_function_name":	"hash_app_entry",
	    "expected_hash":		input.hash || input.target,
	});
    },
    async get_apphub_webapp ( input ) {
	return await this.functions.get_apphub_entry({
	    "dna":			input.dna,
	    "zome":			"apphub_csr",
	    "function":			"get_webapp_entry",
	    "args":			input.target,
	    "hash_function_name":	"hash_webapp_entry",
	    "expected_hash":		input.hash || input.target,
	});
    },
    async get_apphub_webapp_package ( input ) {
	return await this.functions.get_apphub_entry({
	    "dna":			input.dna,
	    "zome":			"apphub_csr",
	    "function":			"get_webapp_package_entry",
	    "args":			input.target,
	    "hash_function_name":	"hash_webapp_package_entry",
	    "expected_hash":		input.hash || input.target,
	});
    },
    async get_apphub_webapp_package_version ( input ) {
	return await this.functions.get_apphub_entry({
	    "dna":			input.dna,
	    "zome":			"apphub_csr",
	    "function":			"get_webapp_package_version_entry",
	    "args":			input.target,
	    "hash_function_name":	"hash_webapp_package_version_entry",
	    "expected_hash":		input.hash || input.target,
	});
    },
    async call_apphub_zome_function ( input ) {
	const apphub			= this.getCellInterface( "apphub", input.dna );

	return await apphub[ input.zome ][ input.function ](
	    input.args
	);
    },
    async get_apphub_entry ( input ) {
	const entry			= await this.functions.call_apphub_zome_function( input );

	this.log.normal("Checking AppHub entry with '%s':", input.hash_function_name, entry );
	const recv_hash			= await this.functions[ input.hash_function_name ](
	    entry
	);

	if ( String(input.expected_hash) !== String(recv_hash) )
	    throw new Error(`Hashes do not match: ${input.expected_hash} !== ${recv_hash}`);

	return entry;
    },
    async get_apphub_webapp_package_versions ( input ) {
	const app			= await this.functions.get_app( input );
	const apphub			= this.getCellInterface( "apphub", app.apphub_hrl.dna );

	return await apphub.apphub_csr.get_webapp_package_versions_sorted(
	    app.apphub_hrl.target
	);
    },
    async get_apphub_webapp_package_version_bundle ( input ) {
	const app_version		= await this.functions.get_app_version( input );
	const apphub			= this.getCellInterface( "apphub", app_version.apphub_hrl.dna );

	const webapp_version		= await this.functions.get_apphub_webapp_package_version({
	    "dna":	app_version.apphub_hrl.dna,
	    "target":	app_version.apphub_hrl.target,
	    "hash":	app_version.apphub_hrl_hash,
	});

	const bundle_bytes		= await apphub.apphub_csr.get_webhapp_bundle(
	    webapp_version.webapp
	);

	const webapp_bundle		= new Bundle( bundle_bytes, "webhapp" );

	const calc_hash			= await this.zomes.mere_memory_api.calculate_hash(
	    bundle_bytes,
	);
	const calc_ui_hash		= await this.zomes.mere_memory_api.calculate_hash(
	    webapp_bundle.ui()
	);
	const calc_happ_hash		= await this.zomes.mere_memory_api.calculate_hash(
	    webapp_bundle.resources[ webapp_bundle.manifest.happ_manifest.bundled ]
	);

	if ( calc_hash !== app_version.bundle_hashes.hash )
	    throw new Error(`Recevied bundle hash does not match expected bundle hash: ${calc_hash} !== ${app_version.bundle_hashes.hash}`);

	return webapp_bundle;
    },
}, {
    "zomes": {
	"mere_memory_api": MereMemoryZomelet,
    },
    "virtual": {
	"cells": {
	    "apphub": AppHubCell,
	    "dnahub": DnaHubCell,
	    "zomehub": ZomeHubCell,
	},
    },
});


export const AppStoreCell		= new CellZomelets({
    "appstore_csr": AppStoreCSRZomelet,
    "mere_memory_api": MereMemoryZomelet,
});


export { MereMemoryZomelet }		from '@spartan-hc/mere-memory-zomelets';
export *				from './types.js';

export default {
    // Zomelets
    AppStoreCSRZomelet,
    MereMemoryZomelet,

    // CellZomelets
    AppStoreCell,
};
