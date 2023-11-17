
import {
    AgentPubKey, HoloHash,
    ActionHash, EntryHash, DnaHash,
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

export const WebHappConfigStruct = {
    "dna":			DnaHash,
    "happ":			ActionHash,
    "gui":			OptionType( ActionHash ),
};



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
}



export const AppStruct = {
    "title":			String,
    "subtitle":			String,
    "description":		String,
    "icon":			EntryHash,
    "publisher":		ActionHash,
    "devhub_address":		WebHappConfigStruct,
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
