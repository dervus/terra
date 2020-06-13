create function now_utc() returns timestamp as $$
    select now() at time zone 'UTC'
$$ language sql;

create function updated_at_trigger_fn() returns trigger as $$
begin
    if (new is distinct from old and new.updated_at is not distinct from old.updated_at) then
        new.updated_at := now_utc();
    end if;
    return new;
end;
$$ language plpgsql;

create table accounts (
    account_id serial4 primary key,
    nick text not null,
    email text not null,
    password_hash text not null,
    access_level int2 not null default 0 check (access_level between 0 and 3),
    created_at timestamp not null default now_utc()
);
create unique index accounts_nick_idx on accounts ((lower(nick)));
create unique index accounts_email_idx on accounts ((lower(email)));

create table sessions (
    session_key bytea primary key,
    account_id int4 not null references accounts on delete cascade,
    created_at timestamp not null default now_utc(),
    last_access_at timestamp not null default now_utc()
);

create type character_status as enum ('pending', 'reviewed', 'rejected', 'finalized');
create type character_gender as enum ('male', 'female');

create table characters (
    character_id serial4 primary key,
    account_id int4 references accounts on delete set null,
    campaign text not null,
    role text,
    status character_status not null,
    gender character_gender not null default 'male',
    race int2 not null check (race between 1 and 255),
    class int2 not null check (class between 1 and 255),
    armor text,
    weapon text,
    traits text[] not null default '{}',
    location text not null,
    name text not null,
    name_extra text check (name_extra is null or trim(both from name_extra) <> ''),
    info_public text check (info_public is null or trim(both from info_public) <> ''),
    info_hidden text check (info_hidden is null or trim(both from info_hidden) <> ''),
    private bool not null default false,
    created_at timestamp not null default now_utc(),
    updated_at timestamp
);
create trigger set_updated_at before update on characters for each row execute function updated_at_trigger_fn();
create unique index characters_name_idx on characters (campaign, (lower(name)));

create table notes (
    note_id serial4 primary key,
    character_id int4 not null references characters on delete cascade,
    account_id int4 references accounts on delete set null,
    contents text not null,
    created_at timestamp not null default now_utc(),
    updated_at timestamp
);
create trigger set_updated_at before update on notes for each row execute function updated_at_trigger_fn();
