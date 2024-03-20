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
}					from '@holochain/apphub-zomelets';
import {
    Publisher,
    App,
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
    "get_apps_for_agent":		true,
    "get_my_apps":			true,
    "get_all_apps":			true,
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
    async get_webapp_package_versions ( input ) {
	const app			= await this.functions.get_app( input );
	const apphub			= this.getCellInterface( "apphub", app.apphub_hrl.dna );

	return await apphub.apphub_csr.get_webapp_package_versions_sorted(
	    app.apphub_hrl.target
	);
    },
    async get_webapp_package_latest_bundle ( input ) {
	const app			= await this.functions.get_app( input );
	const apphub			= this.getCellInterface( "apphub", app.apphub_hrl.dna );

	const versions			= await apphub.apphub_csr.get_webapp_package_versions_sorted(
	    app.apphub_hrl.target
	);

	return await apphub.apphub_csr.get_webhapp_bundle(
	    versions[0].webapp
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
    async call_apphub_zome_function ( input ) {
	const apphub			= this.getCellInterface( "apphub", input.dna );

	return await apphub[ input.zome ][ input.function ](
	    input.args
	);
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
