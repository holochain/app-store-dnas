
import {
    HoloHash, AnyDhtHash,
    AgentPubKey, DnaHash,
    ActionHash, EntryHash,
}					from '@spartan-hc/holo-hash';
import {
    ScopedEntity,
    intoStruct,
    AnyType, OptionType,
    VecType, MapType,
}					from '@spartan-hc/caps-entities';



export const LocationTripletStruct = {
    "country":			String,
    "region":			String,
    "city":			String,
};

export const WebAddressStruct = {
    "url":			String,
    "context":			OptionType( String ),
};

export const DeprecationNoticeStruct = {
    "message":			String,
    "recommended_alternatives":	OptionType( VecType( ActionHash ) ),
};

export const HRLStruct = {
    "dna":			DnaHash,
    "target":			AnyDhtHash,
}



export const PublisherStruct = {
    "name":			String,
    "location":			LocationTripletStruct,
    "website":			WebAddressStruct,
    "icon":			EntryHash,
    "editors":			VecType( AgentPubKey ),

    // common fields
    "author":			AgentPubKey,
    "published_at":		Number,
    "last_updated":		Number,
    "metadata":			MapType( String, AnyType ),

    // optional
    "description":		OptionType( String ),
    "email":			OptionType( String ),
    "deprecation":		OptionType( DeprecationNoticeStruct ),
};

export function PublisherEntry ( entry ) {
    return intoStruct( entry, PublisherStruct );
}

export class Publisher extends ScopedEntity {
    static STRUCT		= PublisherStruct;

    async $update ( changes ) {
	const result		= await this.zome.update_publisher({
	    "base": this.$action,
	    "properties": changes,
	});

	super.$update( result );

	return this;
    }

    async $deprecate ( message ) {
	const result		= await this.zome.deprecate_publisher({
	    "base": this.$action,
	    message,
	});

	super.$update( result );

	return this;
    }

    async $undeprecate () {
	const result		= await this.zome.undeprecate_publisher({
	    "base": this.$action,
	});

	super.$update( result );

	return this;
    }

}



export const AppStruct = {
    "title":			String,
    "subtitle":			String,
    "description":		String,
    "icon":			EntryHash,
    "publisher":		ActionHash,
    "apphub_hrl":		HRLStruct,
    "editors":			VecType( AgentPubKey ),

    // common fields
    "author":			AgentPubKey,
    "published_at":		Number,
    "last_updated":		Number,
    "metadata":			MapType( String, AnyType ),

    // optional
    "deprecation":		OptionType( DeprecationNoticeStruct ),
};

export function AppEntry ( entry ) {
    return intoStruct( entry, AppStruct );
}

export class App extends ScopedEntity {
    static STRUCT		= AppStruct;

    async $update ( changes ) {
	const result		= await this.zome.update_app({
	    "base": this.$action,
	    "properties": changes,
	});

	super.$update( result );

	return this;
    }

    async $getWebAppPackage () {
	return await this.zome.get_webapp_package( this.$id );
    }

    async $getWebAppPackageVersions () {
	return await this.zome.get_webapp_package_versions( this.$id );
    }

    async $getLatestBundle () {
	return await this.zome.get_webapp_package_latest_bundle( this.$id );
    }

    async $getModeratedState ( group_id ) {
	return await this.zome.get_moderated_state({
	    group_id,
	    "app_id": this.$id,
	});
    }

    async $updateModeratedState ( group_id, input ) {
	return await this.zome.update_moderated_state({
	    group_id,
	    "app_id": this.$id,
	    "message": input.message,
	    "metadata": input.metadata,
	});
    }

    async $deprecate ( message ) {
	const result		= await this.zome.deprecate_app({
	    "base": this.$action,
	    message,
	});

	super.$update( result );

	return this;
    }

    async $undeprecate () {
	const result		= await this.zome.undeprecate_app({
	    "base": this.$action,
	});

	super.$update( result );

	return this;
    }

}



export const GroupStruct = {
    "admins":			VecType( AgentPubKey ),
    "members":			VecType( AgentPubKey ),
    "deleted":			OptionType( Boolean ),

    // common fields
    "published_at":		Number,
    "last_updated":		Number,
    "metadata":			MapType( String, AnyType ),
};

export function GroupEntry ( entry ) {
    return intoStruct( entry, GroupStruct );
}

export class Group extends ScopedEntity {
    static STRUCT		= GroupStruct;

    async $getAllApps () {
	return await this.zome.viewpoint_get_all_apps( this.$id );
    }

    async $getAllRemovedApps () {
	return await this.zome.viewpoint_get_all_removed_apps( this.$id );
    }

    async $getAppModeratedState ( app_id ) {
	return await this.zome.get_moderated_state({
	    "group_id": this.$id,
	    app_id,
	});
    }

    async $getAppModeratedActions ( app_id ) {
	return await this.zome.get_moderator_actions({
	    "group_id": this.$id,
	    app_id,
	});
    }

    async $removeApp ( app_id, message ) {
	const ma_state		= await this.$getAppModeratedState( app_id );
	const metadata		= Object.assign( {}, ma_state?.metastate, {
	    "remove": true,
	});

	return await this.zome.update_moderated_state({
	    "group_id": this.$id,
	    app_id,
	    message,
	    metadata,
	});
    }

    async $unremoveApp ( app_id, message ) {
	const ma_state		= await this.$getAppModeratedState( app_id );
	const metadata		= Object.assign( {}, ma_state?.metastate, {
	    "remove": false,
	});

	return await this.zome.update_moderated_state({
	    "group_id": this.$id,
	    app_id,
	    message,
	    metadata,
	});
    }
}



export default {
    PublisherStruct,
    PublisherEntry,
    Publisher,

    AppStruct,
    AppEntry,
    App,

    GroupStruct,
    GroupEntry,
    Group,
};