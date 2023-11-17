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
    Publisher,
    App,
    Group,
}					from './types.js';


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
    "get_moderator_actions":		true,
    "get_moderated_state":		true,
    "update_moderated_state":		true,
    "viewpoint_get_all_apps":		true,
    "viewpoint_get_all_removed_apps":	true,


    //
    // Virtual functions
    //
}, {
    "zomes": {
	"mere_memory_api": MereMemoryZomelet,
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
