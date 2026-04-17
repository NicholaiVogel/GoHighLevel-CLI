pub mod commands;
pub mod output;

use std::io::{self, Read};

use clap::{CommandFactory, Parser, error::ErrorKind};
use commands::{
    AuthCommand, AuthPitAddArgs, AuthPitCommand, CalendarsCommand, Cli, Command, CommandsCommand,
    ConfigCommand, ContactsCommand, ConversationsCommand, EndpointsCommand, ErrorsCommand,
    LocationsCommand, OpportunitiesCommand, PipelinesCommand, ProfilePolicyCommand,
    ProfilePolicySetArgs, ProfilesCommand, RawCommand, SmokeCommand, SmokeRunArgs,
};
use ghl::{GhlError, Result};
use output::{print_error, print_success};
use serde::Serialize;

pub fn run_cli() -> i32 {
    match Cli::try_parse() {
        Ok(cli) => match execute(cli) {
            Ok(()) => 0,
            Err(error) => {
                print_error(&error);
                error.exit_code()
            }
        },
        Err(error) => handle_clap_error(error),
    }
}

fn handle_clap_error(error: clap::Error) -> i32 {
    match error.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
            let _ = error.print();
            error.exit_code()
        }
        _ => {
            let ghl_error = GhlError::Validation {
                message: error.to_string(),
            };
            print_error(&ghl_error);
            ghl_error.exit_code()
        }
    }
}

fn execute(cli: Cli) -> Result<()> {
    let format = cli.format;
    let pretty = cli.pretty;
    let config_dir = cli.config_dir.clone();
    let selected_profile = cli.profile.clone();
    let selected_location = cli.location.clone();
    let selected_company = cli.company.clone();
    let dry_run = cli.dry_run;

    if cli.offline && !is_local_command(&cli.command, dry_run) {
        return Err(GhlError::OfflineBlocked {
            command: command_name(&cli.command),
        });
    }

    match cli.command {
        Command::Commands(CommandsCommand::Schema) => {
            print_success(ghl::command_schema(), format, pretty)
        }
        Command::Config(ConfigCommand::Path) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            print_success(paths, format, pretty)
        }
        Command::Config(ConfigCommand::Show) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            print_success(
                ghl::profiles::redacted_config_with_profiles(paths)?,
                format,
                pretty,
            )
        }
        Command::Config(ConfigCommand::Doctor) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            print_success(ghl::config::config_doctor(&paths), format, pretty)
        }
        Command::Auth(AuthCommand::Pit(AuthPitCommand::Add(args))) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            let profile = selected_profile.as_deref().unwrap_or("default");
            let token = read_pit_token(&args)?;
            print_success(
                ghl::add_pit(
                    &paths,
                    profile,
                    token,
                    args.location.or(selected_location.clone()),
                    args.company.or(selected_company.clone()),
                    args.set_default,
                )?,
                format,
                pretty,
            )
        }
        Command::Auth(AuthCommand::Pit(AuthPitCommand::Validate)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            print_success(
                ghl::validate_pit(&paths, selected_profile.as_deref())?,
                format,
                pretty,
            )
        }
        Command::Auth(AuthCommand::Pit(AuthPitCommand::ListLocal)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            print_success(ghl::list_local_pits(&paths)?, format, pretty)
        }
        Command::Auth(AuthCommand::Pit(AuthPitCommand::RemoveLocal(args))) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            print_success(
                ghl::remove_local_pit(&paths, &args.credential_ref)?,
                format,
                pretty,
            )
        }
        Command::Auth(AuthCommand::Status) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            print_success(
                ghl::auth_status(&paths, selected_profile.as_deref())?,
                format,
                pretty,
            )
        }
        Command::Profiles(ProfilesCommand::List) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            let profiles = ghl::load_profiles(&paths)?;
            print_success(profiles.list(), format, pretty)
        }
        Command::Profiles(ProfilesCommand::Show(args)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            let profiles = ghl::load_profiles(&paths)?;
            print_success(profiles.get_required(&args.name)?, format, pretty)
        }
        Command::Profiles(ProfilesCommand::SetDefault(args)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            print_success(
                ghl::set_default_profile(&paths, &args.name)?,
                format,
                pretty,
            )
        }
        Command::Profiles(ProfilesCommand::SetDefaultCompany(args)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            print_success(
                ghl::set_default_company(&paths, &args.name, &args.company_id)?,
                format,
                pretty,
            )
        }
        Command::Profiles(ProfilesCommand::SetDefaultLocation(args)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            print_success(
                ghl::set_default_location(&paths, &args.name, &args.location_id)?,
                format,
                pretty,
            )
        }
        Command::Profiles(ProfilesCommand::Policy(ProfilePolicyCommand::Show(args))) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            let profiles = ghl::load_profiles(&paths)?;
            print_success(&profiles.get_required(&args.name)?.policy, format, pretty)
        }
        Command::Profiles(ProfilesCommand::Policy(ProfilePolicyCommand::Set(args))) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            let policy = set_policy(&paths, args)?;
            print_success(policy, format, pretty)
        }
        Command::Profiles(ProfilesCommand::Policy(ProfilePolicyCommand::Reset(args))) => {
            if !args.yes {
                return Err(GhlError::ConfirmationRequired);
            }
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            let mut profiles = ghl::load_profiles(&paths)?;
            let profile = profiles.get_required_mut(&args.name)?;
            profile.policy = ghl::ProfilePolicy::default();
            let policy = profile.policy.clone();
            ghl::save_profiles(&paths, &profiles)?;
            print_success(policy, format, pretty)
        }
        Command::Errors(ErrorsCommand::List) => {
            print_success(ghl::error_definitions(), format, pretty)
        }
        Command::Errors(ErrorsCommand::Show(args)) => {
            let definition = ghl::find_error_definition(&args.error_code).ok_or_else(|| {
                GhlError::ErrorCodeNotFound {
                    code: args.error_code.clone(),
                }
            })?;
            print_success(definition, format, pretty)
        }
        Command::Endpoints(EndpointsCommand::List) => {
            let manifest = ghl::bundled_manifest()?;
            print_success(manifest.endpoints, format, pretty)
        }
        Command::Endpoints(EndpointsCommand::Show(args)) => {
            let manifest = ghl::bundled_manifest()?;
            let endpoint = ghl::endpoints::find_endpoint(&manifest, &args.endpoint_key)
                .ok_or_else(|| GhlError::EndpointNotFound {
                    key: args.endpoint_key.clone(),
                })?;
            print_success(endpoint, format, pretty)
        }
        Command::Endpoints(EndpointsCommand::Coverage) => {
            let manifest = ghl::bundled_manifest()?;
            print_success(ghl::endpoints::endpoint_coverage(&manifest), format, pretty)
        }
        Command::Raw(RawCommand::Request(args)) => {
            let request = ghl::RawGetRequest {
                surface: args.surface.into(),
                path: args.path,
                auth_class: args.auth.into(),
                include_body: args.include_body,
            };
            if dry_run.is_some() {
                print_success(RawDryRun::from_request(&request), format, pretty)
            } else {
                let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
                print_success(
                    ghl::raw_get(&paths, selected_profile.as_deref(), request)?,
                    format,
                    pretty,
                )
            }
        }
        Command::Locations(LocationsCommand::Get(args)) => {
            if dry_run.is_some() {
                print_success(
                    ghl::get_location_dry_run(&args.location_id)?,
                    format,
                    pretty,
                )
            } else {
                let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
                print_success(
                    ghl::get_location(&paths, selected_profile.as_deref(), &args.location_id)?,
                    format,
                    pretty,
                )
            }
        }
        Command::Locations(LocationsCommand::List(args)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            let options = ghl::LocationSearchOptions {
                company_id: args.company.or(selected_company.clone()),
                email: None,
                skip: args.skip,
                limit: args.limit,
                order: args.order.into(),
            };
            if dry_run.is_some() {
                print_success(
                    ghl::locations_search_dry_run(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        options,
                    )?,
                    format,
                    pretty,
                )
            } else {
                print_success(
                    ghl::list_locations(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        options,
                    )?,
                    format,
                    pretty,
                )
            }
        }
        Command::Locations(LocationsCommand::Search(args)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            let options = ghl::LocationSearchOptions {
                company_id: args.company.or(selected_company.clone()),
                email: Some(args.query),
                skip: args.skip,
                limit: args.limit,
                order: args.order.into(),
            };
            if dry_run.is_some() {
                print_success(
                    ghl::locations_search_dry_run(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        options,
                    )?,
                    format,
                    pretty,
                )
            } else {
                print_success(
                    ghl::search_locations(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        options,
                    )?,
                    format,
                    pretty,
                )
            }
        }
        Command::Contacts(ContactsCommand::List(args)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            let options = ghl::ContactListOptions {
                limit: args.limit,
                start_after_id: args.start_after_id,
                start_after: args.start_after,
            };
            if dry_run.is_some() {
                print_success(
                    ghl::contacts_list_dry_run(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        options,
                    )?,
                    format,
                    pretty,
                )
            } else {
                print_success(
                    ghl::list_contacts(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        options,
                    )?,
                    format,
                    pretty,
                )
            }
        }
        Command::Contacts(ContactsCommand::Search(args)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            let options = ghl::ContactSearchOptions {
                query: args.query,
                email: args.email,
                phone: args.phone,
                limit: args.limit,
                start_after_id: args.start_after_id,
                start_after: args.start_after,
            };
            if dry_run.is_some() {
                print_success(
                    ghl::contacts_search_dry_run(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        options,
                    )?,
                    format,
                    pretty,
                )
            } else {
                print_success(
                    ghl::search_contacts(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        options,
                    )?,
                    format,
                    pretty,
                )
            }
        }
        Command::Contacts(ContactsCommand::Get(args)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            if dry_run.is_some() {
                print_success(
                    ghl::get_contact_dry_run(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        &args.contact_id,
                    )?,
                    format,
                    pretty,
                )
            } else {
                print_success(
                    ghl::get_contact(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        &args.contact_id,
                    )?,
                    format,
                    pretty,
                )
            }
        }
        Command::Conversations(ConversationsCommand::Search(args)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            let options = ghl::ConversationSearchOptions {
                contact_id: args.contact,
                query: args.query,
                status: args.status.into(),
                assigned_to: args.assigned_to,
                limit: args.limit,
                last_message_type: args.last_message_type,
                start_after_date: args.start_after_date,
            };
            if dry_run.is_some() {
                print_success(
                    ghl::conversations_search_dry_run(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        options,
                    )?,
                    format,
                    pretty,
                )
            } else {
                print_success(
                    ghl::search_conversations(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        options,
                    )?,
                    format,
                    pretty,
                )
            }
        }
        Command::Conversations(ConversationsCommand::Get(args)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            if dry_run.is_some() {
                print_success(
                    ghl::get_conversation_dry_run(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        &args.conversation_id,
                    )?,
                    format,
                    pretty,
                )
            } else {
                print_success(
                    ghl::get_conversation(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        &args.conversation_id,
                    )?,
                    format,
                    pretty,
                )
            }
        }
        Command::Conversations(ConversationsCommand::Messages(args)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            let options = ghl::ConversationMessagesOptions {
                limit: args.limit,
                last_message_id: args.last_message_id,
                message_type: args.message_type,
            };
            if dry_run.is_some() {
                print_success(
                    ghl::conversation_messages_dry_run(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        &args.conversation_id,
                        options,
                    )?,
                    format,
                    pretty,
                )
            } else {
                print_success(
                    ghl::get_conversation_messages(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        &args.conversation_id,
                        options,
                    )?,
                    format,
                    pretty,
                )
            }
        }
        Command::Pipelines(PipelinesCommand::List) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            if dry_run.is_some() {
                print_success(
                    ghl::pipelines_list_dry_run(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                    )?,
                    format,
                    pretty,
                )
            } else {
                print_success(
                    ghl::list_pipelines(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                    )?,
                    format,
                    pretty,
                )
            }
        }
        Command::Pipelines(PipelinesCommand::Get(args)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            if dry_run.is_some() {
                print_success(
                    ghl::get_pipeline_dry_run(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        &args.pipeline_id,
                    )?,
                    format,
                    pretty,
                )
            } else {
                print_success(
                    ghl::get_pipeline(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        &args.pipeline_id,
                    )?,
                    format,
                    pretty,
                )
            }
        }
        Command::Opportunities(OpportunitiesCommand::Search(args)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            let options = ghl::OpportunitySearchOptions {
                query: args.query,
                pipeline_id: args.pipeline,
                pipeline_stage_id: args.stage,
                contact_id: args.contact,
                status: args.status.map(Into::into),
                assigned_to: args.assigned_to,
                limit: args.limit,
                page: args.page,
                start_after_id: args.start_after_id,
                start_after: args.start_after,
            };
            if dry_run.is_some() {
                print_success(
                    ghl::opportunities_search_dry_run(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        options,
                    )?,
                    format,
                    pretty,
                )
            } else {
                print_success(
                    ghl::search_opportunities(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        options,
                    )?,
                    format,
                    pretty,
                )
            }
        }
        Command::Opportunities(OpportunitiesCommand::Get(args)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            if dry_run.is_some() {
                print_success(
                    ghl::get_opportunity_dry_run(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        &args.opportunity_id,
                    )?,
                    format,
                    pretty,
                )
            } else {
                print_success(
                    ghl::get_opportunity(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        &args.opportunity_id,
                    )?,
                    format,
                    pretty,
                )
            }
        }

        Command::Calendars(CalendarsCommand::List(args)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            let options = ghl::CalendarListOptions {
                group_id: args.group,
                show_drafted: args.show_drafted,
            };
            if dry_run.is_some() {
                print_success(
                    ghl::calendars_list_dry_run(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        options,
                    )?,
                    format,
                    pretty,
                )
            } else {
                print_success(
                    ghl::list_calendars(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        options,
                    )?,
                    format,
                    pretty,
                )
            }
        }
        Command::Calendars(CalendarsCommand::Get(args)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            if dry_run.is_some() {
                print_success(
                    ghl::get_calendar_dry_run(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        &args.calendar_id,
                    )?,
                    format,
                    pretty,
                )
            } else {
                print_success(
                    ghl::get_calendar(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        &args.calendar_id,
                    )?,
                    format,
                    pretty,
                )
            }
        }
        Command::Calendars(CalendarsCommand::Events(args)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            let options = ghl::CalendarEventsOptions {
                calendar_id: args.calendar,
                group_id: args.group,
                user_id: args.user,
                from: args.from,
                to: args.to,
                date: args.date,
            };
            if dry_run.is_some() {
                print_success(
                    ghl::calendar_events_dry_run(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        options,
                    )?,
                    format,
                    pretty,
                )
            } else {
                print_success(
                    ghl::list_calendar_events(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        options,
                    )?,
                    format,
                    pretty,
                )
            }
        }
        Command::Calendars(CalendarsCommand::FreeSlots(args)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            let options = ghl::CalendarFreeSlotsOptions {
                calendar_id: args.calendar,
                date: args.date,
                timezone: args.timezone,
                user_id: args.user,
                enable_look_busy: args.enable_look_busy,
            };
            if dry_run.is_some() {
                print_success(
                    ghl::calendar_free_slots_dry_run(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        options,
                    )?,
                    format,
                    pretty,
                )
            } else {
                print_success(
                    ghl::get_calendar_free_slots(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        options,
                    )?,
                    format,
                    pretty,
                )
            }
        }
        Command::Smoke(SmokeCommand::Run(args)) => {
            let paths = ghl::resolve_paths_from_env(config_dir.as_deref())?;
            let options = smoke_options(args);
            if dry_run.is_some() {
                print_success(
                    ghl::smoke_run_dry_run(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        selected_company.as_deref(),
                        options,
                    ),
                    format,
                    pretty,
                )
            } else {
                print_success(
                    ghl::smoke_run(
                        &paths,
                        selected_profile.as_deref(),
                        selected_location.as_deref(),
                        selected_company.as_deref(),
                        options,
                    ),
                    format,
                    pretty,
                )
            }
        }
        Command::Completions(args) => {
            let mut command = Cli::command();
            let shell: clap_complete::Shell = args.shell.into();
            clap_complete::generate(shell, &mut command, "ghl", &mut io::stdout());
            Ok(())
        }
        Command::Man => {
            let mut command = Cli::command();
            println!("{}", command.render_long_help());
            Ok(())
        }
    }
}

#[derive(Debug, Serialize)]
struct RawDryRun {
    method: &'static str,
    surface: String,
    path: String,
    auth_class: &'static str,
    include_body: bool,
    network: bool,
}

impl RawDryRun {
    fn from_request(request: &ghl::RawGetRequest) -> Self {
        Self {
            method: "GET",
            surface: request.surface.as_str().to_owned(),
            path: request.path.clone(),
            auth_class: "pit",
            include_body: request.include_body,
            network: false,
        }
    }
}

fn read_pit_token(args: &AuthPitAddArgs) -> Result<String> {
    let source_count =
        args.token.is_some() as u8 + args.token_env.is_some() as u8 + args.token_stdin as u8;
    if source_count == 0 {
        return Err(GhlError::MissingTokenInput);
    }
    if source_count > 1 {
        return Err(GhlError::ConflictingTokenInput);
    }

    let token = if let Some(token) = &args.token {
        token.clone()
    } else if let Some(env_name) = &args.token_env {
        std::env::var(env_name).map_err(|_| GhlError::Validation {
            message: format!("token environment variable `{env_name}` is not set"),
        })?
    } else {
        let mut token = String::new();
        io::stdin()
            .read_to_string(&mut token)
            .map_err(|source| GhlError::FileRead {
                path: "<stdin>".into(),
                source,
            })?;
        token
    };

    let token = token
        .trim_matches(|character| character == '\n' || character == '\r')
        .to_owned();
    if token.is_empty() {
        return Err(GhlError::MissingTokenInput);
    }

    Ok(token)
}

fn set_policy(paths: &ghl::ConfigPaths, args: ProfilePolicySetArgs) -> Result<ghl::ProfilePolicy> {
    let patch = ghl::ProfilePolicyPatch {
        agent_mode: args.agent_mode,
        default_dry_run: args.default_dry_run,
        allow_destructive: args.allow_destructive,
        allow_messaging: args.allow_messaging,
        allow_payment_actions: args.allow_payment_actions,
        allow_public_links: args.allow_public_links,
        allow_workflow_publish: args.allow_workflow_publish,
        allow_phone_purchase: args.allow_phone_purchase,
        allow_private_integration_token_create: args.allow_private_integration_token_create,
    };

    if patch == ghl::ProfilePolicyPatch::default() {
        return Err(GhlError::Validation {
            message: "policy set did not include any changes".to_owned(),
        });
    }

    let mut profiles = ghl::load_profiles(paths)?;
    let profile = profiles.get_required_mut(&args.name)?;
    profile.policy.apply_patch(&patch);
    let policy = profile.policy.clone();
    ghl::save_profiles(paths, &profiles)?;

    Ok(policy)
}

fn smoke_options(args: SmokeRunArgs) -> ghl::SmokeRunOptions {
    ghl::SmokeRunOptions {
        limit: args.limit,
        skip_optional: args.skip_optional,
        contact_query: args.contact_query,
        contact_email: args.contact_email,
        contact_phone: args.contact_phone,
        contact_id: args.contact_id,
        conversation_id: args.conversation_id,
        pipeline_id: args.pipeline_id,
        opportunity_id: args.opportunity_id,
        calendar_id: args.calendar_id,
        calendar_date: args.calendar_date,
        calendar_timezone: args.calendar_timezone,
    }
}

fn is_local_command(command: &Command, dry_run: Option<commands::DryRunMode>) -> bool {
    if dry_run.is_some() {
        return true;
    }

    !matches!(
        command,
        Command::Auth(AuthCommand::Pit(AuthPitCommand::Validate))
            | Command::Raw(_)
            | Command::Locations(_)
            | Command::Contacts(_)
            | Command::Conversations(_)
            | Command::Pipelines(_)
            | Command::Opportunities(_)
            | Command::Calendars(_)
            | Command::Smoke(_)
    )
}

fn command_name(command: &Command) -> String {
    match command {
        Command::Commands(_) => "commands".to_owned(),
        Command::Config(_) => "config".to_owned(),
        Command::Auth(_) => "auth".to_owned(),
        Command::Profiles(_) => "profiles".to_owned(),
        Command::Errors(_) => "errors".to_owned(),
        Command::Endpoints(_) => "endpoints".to_owned(),
        Command::Raw(_) => "raw".to_owned(),
        Command::Locations(_) => "locations".to_owned(),
        Command::Contacts(_) => "contacts".to_owned(),
        Command::Conversations(_) => "conversations".to_owned(),
        Command::Pipelines(_) => "pipelines".to_owned(),
        Command::Opportunities(_) => "opportunities".to_owned(),
        Command::Calendars(_) => "calendars".to_owned(),
        Command::Smoke(_) => "smoke".to_owned(),
        Command::Completions(_) => "completions".to_owned(),
        Command::Man => "man".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{Command, CommandsCommand, ConfigCommand, RawCommand, RawRequestArgs};

    #[test]
    fn phase_one_commands_are_local_for_offline_mode() {
        assert!(is_local_command(
            &Command::Commands(CommandsCommand::Schema),
            None
        ));
        assert!(is_local_command(
            &Command::Config(ConfigCommand::Path),
            None
        ));
    }

    #[test]
    fn raw_is_remote_unless_dry_run_is_set() {
        let command = Command::Raw(RawCommand::Request(RawRequestArgs {
            surface: crate::commands::RawSurface::Services,
            method: crate::commands::RawMethod::Get,
            path: "/locations/loc_123".to_owned(),
            auth: crate::commands::RawAuthClass::Pit,
            include_body: false,
        }));

        assert!(!is_local_command(&command, None));
        assert!(is_local_command(
            &command,
            Some(crate::commands::DryRunMode::Local)
        ));
    }

    #[test]
    fn command_name_returns_group_name() {
        assert_eq!(
            command_name(&Command::Commands(CommandsCommand::Schema)),
            "commands"
        );
    }
}
