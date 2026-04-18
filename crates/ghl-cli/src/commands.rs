use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "ghl-cli",
    bin_name = "ghl",
    version,
    about = "Unofficial Go High Level CLI for humans, scripts, and AI agents.",
    long_about = "Unofficial Go High Level CLI for humans, scripts, and AI agents.\n\nThe current implementation provides the local CLI spine, profile persistence, local PIT credential storage, guarded read-only PIT validation, raw GET, typed CRM reads, and read-only smoke validation."
)]
pub struct Cli {
    #[arg(long, global = true, env = "GHL_CLI_PROFILE")]
    pub profile: Option<String>,

    #[arg(long, global = true, env = "GHL_CLI_LOCATION_ID")]
    pub location: Option<String>,

    #[arg(long, global = true, env = "GHL_CLI_COMPANY_ID")]
    pub company: Option<String>,

    #[arg(long, global = true, env = "GHL_CLI_CONFIG_DIR")]
    pub config_dir: Option<PathBuf>,

    #[arg(long, global = true, value_enum, env = "GHL_CLI_FORMAT", default_value_t = OutputFormat::Json)]
    pub format: OutputFormat,

    #[arg(long, global = true, default_value_t = false)]
    pub pretty: bool,

    #[arg(long, global = true, default_value_t = false)]
    pub quiet: bool,

    #[arg(long, global = true, default_value_t = false)]
    pub verbose: bool,

    #[arg(long, global = true, value_enum, num_args = 0..=1, require_equals = true, default_missing_value = "local")]
    pub dry_run: Option<DryRunMode>,

    #[arg(long, global = true, default_value_t = false)]
    pub yes: bool,

    #[arg(long, global = true, env = "GHL_CLI_NO_CACHE", default_value_t = false)]
    pub no_cache: bool,

    #[arg(long, global = true, env = "GHL_CLI_TIMEOUT")]
    pub timeout: Option<String>,

    #[arg(long, global = true, default_value_t = false)]
    pub offline: bool,

    #[arg(long, global = true)]
    pub lock_timeout: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Table,
    Ndjson,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum DryRunMode {
    Local,
    Validated,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(subcommand)]
    Commands(CommandsCommand),

    #[command(subcommand)]
    Config(ConfigCommand),

    #[command(subcommand)]
    Auth(AuthCommand),

    #[command(subcommand)]
    Profiles(ProfilesCommand),

    #[command(subcommand)]
    Errors(ErrorsCommand),

    #[command(subcommand)]
    Endpoints(EndpointsCommand),

    Doctor(DoctorArgs),

    Capabilities(CapabilitiesArgs),

    #[command(subcommand)]
    Audit(AuditCommand),

    #[command(subcommand)]
    Idempotency(IdempotencyCommand),

    #[command(subcommand)]
    Raw(RawCommand),

    #[command(subcommand)]
    Locations(LocationsCommand),

    #[command(subcommand)]
    Contacts(ContactsCommand),

    #[command(subcommand)]
    Conversations(ConversationsCommand),

    #[command(subcommand)]
    Pipelines(PipelinesCommand),

    #[command(subcommand)]
    Opportunities(OpportunitiesCommand),

    #[command(subcommand)]
    Calendars(CalendarsCommand),

    #[command(subcommand)]
    Appointments(AppointmentsCommand),

    #[command(subcommand)]
    Users(UsersCommand),

    #[command(subcommand)]
    Teams(TeamsCommand),

    #[command(subcommand)]
    Smoke(SmokeCommand),

    Completions(CompletionsArgs),

    Man,
}

#[derive(Debug, Subcommand)]
pub enum CommandsCommand {
    Schema,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    Path,
    Show,
    Doctor,
}

#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    #[command(subcommand)]
    Pit(AuthPitCommand),
    Status,
}

#[derive(Debug, Subcommand)]
pub enum AuthPitCommand {
    Add(AuthPitAddArgs),
    Validate,
    ListLocal,
    RemoveLocal(AuthPitRemoveArgs),
}

#[derive(Debug, Args)]
pub struct AuthPitAddArgs {
    #[arg(long)]
    pub token: Option<String>,

    #[arg(long)]
    pub token_env: Option<String>,

    #[arg(long, default_value_t = false)]
    pub token_stdin: bool,

    #[arg(long)]
    pub location: Option<String>,

    #[arg(long)]
    pub company: Option<String>,

    #[arg(long, default_value_t = true)]
    pub set_default: bool,
}

#[derive(Debug, Args)]
pub struct AuthPitRemoveArgs {
    pub credential_ref: String,
}

#[derive(Debug, Subcommand)]
pub enum ProfilesCommand {
    List,
    Show(ProfileNameArgs),
    SetDefault(ProfileNameArgs),
    SetDefaultCompany(ProfileSetDefaultCompanyArgs),
    SetDefaultLocation(ProfileSetDefaultLocationArgs),
    #[command(subcommand)]
    Policy(ProfilePolicyCommand),
}

#[derive(Debug, Args)]
pub struct ProfileNameArgs {
    pub name: String,
}

#[derive(Debug, Args)]
pub struct ProfileSetDefaultLocationArgs {
    pub name: String,
    pub location_id: String,
}

#[derive(Debug, Args)]
pub struct ProfileSetDefaultCompanyArgs {
    pub name: String,
    pub company_id: String,
}

#[derive(Debug, Subcommand)]
pub enum ProfilePolicyCommand {
    Show(ProfileNameArgs),
    Set(ProfilePolicySetArgs),
    Reset(ProfilePolicyResetArgs),
}

#[derive(Debug, Args)]
pub struct ProfilePolicySetArgs {
    pub name: String,

    #[arg(long)]
    pub agent_mode: Option<bool>,

    #[arg(long)]
    pub default_dry_run: Option<bool>,

    #[arg(long)]
    pub allow_destructive: Option<bool>,

    #[arg(long)]
    pub allow_messaging: Option<bool>,

    #[arg(long)]
    pub allow_payment_actions: Option<bool>,

    #[arg(long)]
    pub allow_public_links: Option<bool>,

    #[arg(long)]
    pub allow_workflow_publish: Option<bool>,

    #[arg(long)]
    pub allow_phone_purchase: Option<bool>,

    #[arg(long)]
    pub allow_private_integration_token_create: Option<bool>,
}

#[derive(Debug, Args)]
pub struct ProfilePolicyResetArgs {
    pub name: String,

    #[arg(long, default_value_t = false)]
    pub yes: bool,
}

#[derive(Debug, Subcommand)]
pub enum ErrorsCommand {
    List,
    Show(ErrorShowArgs),
}

#[derive(Debug, Args)]
pub struct ErrorShowArgs {
    pub error_code: String,
}

#[derive(Debug, Subcommand)]
pub enum EndpointsCommand {
    List,
    Show(EndpointShowArgs),
    Coverage,
}

#[derive(Debug, Args)]
pub struct DoctorArgs {
    #[command(subcommand)]
    pub command: Option<DoctorCommand>,
}

#[derive(Debug, Subcommand)]
pub enum DoctorCommand {
    Api(DoctorApiArgs),
    Endpoint(DoctorEndpointArgs),
    Bundle(DoctorBundleArgs),
}

#[derive(Debug, Args)]
pub struct DoctorApiArgs {
    #[arg(long, default_value_t = 1)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct DoctorEndpointArgs {
    pub endpoint_key: String,
}

#[derive(Debug, Args)]
pub struct DoctorBundleArgs {
    #[arg(long)]
    pub out: PathBuf,

    #[arg(long, alias = "redact", default_value_t = false)]
    pub redacted: bool,
}

#[derive(Debug, Args)]
pub struct CapabilitiesArgs {
    #[command(subcommand)]
    pub command: Option<CapabilitiesCommand>,
}

#[derive(Debug, Subcommand)]
pub enum CapabilitiesCommand {
    List,
    Check(CapabilityCheckArgs),
    Command(CapabilityCheckArgs),
}

#[derive(Debug, Args)]
pub struct CapabilityCheckArgs {
    pub capability: String,
}

#[derive(Debug, Subcommand)]
pub enum AuditCommand {
    List(AuditListArgs),
    Show(AuditShowArgs),
    Export(AuditExportArgs),
}

#[derive(Debug, Args)]
pub struct AuditListArgs {
    /// Inclusive RFC3339 datetime or Unix milliseconds lower bound.
    #[arg(long)]
    pub from: Option<String>,

    /// Inclusive RFC3339 datetime or Unix milliseconds upper bound.
    #[arg(long)]
    pub to: Option<String>,

    #[arg(long)]
    pub action: Option<String>,

    #[arg(long)]
    pub resource: Option<String>,

    #[arg(long, default_value_t = 50)]
    pub limit: usize,
}

#[derive(Debug, Args)]
pub struct AuditShowArgs {
    pub entry_id: String,
}

#[derive(Debug, Args)]
pub struct AuditExportArgs {
    /// Inclusive RFC3339 datetime or Unix milliseconds lower bound.
    #[arg(long)]
    pub from: Option<String>,

    /// Inclusive RFC3339 datetime or Unix milliseconds upper bound.
    #[arg(long)]
    pub to: Option<String>,

    #[arg(long)]
    pub action: Option<String>,

    #[arg(long)]
    pub resource: Option<String>,

    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum IdempotencyCommand {
    List,
    Show(IdempotencyShowArgs),
    Clear(IdempotencyClearArgs),
}

#[derive(Debug, Args)]
pub struct IdempotencyShowArgs {
    pub key: String,
}

#[derive(Debug, Args)]
pub struct IdempotencyClearArgs {
    pub key: String,

    #[arg(long, default_value_t = false)]
    pub yes: bool,
}

#[derive(Debug, Subcommand)]
pub enum RawCommand {
    Request(RawRequestArgs),
}

#[derive(Debug, Args)]
pub struct RawRequestArgs {
    #[arg(long, value_enum)]
    pub surface: RawSurface,

    #[arg(long, value_enum)]
    pub method: RawMethod,

    #[arg(long)]
    pub path: String,

    #[arg(long, value_enum, default_value_t = RawAuthClass::Pit)]
    pub auth: RawAuthClass,

    #[arg(long, default_value_t = false)]
    pub include_body: bool,
}

#[derive(Debug, Subcommand)]
pub enum LocationsCommand {
    Get(LocationGetArgs),
    List(LocationListArgs),
    Search(LocationSearchArgs),
}

#[derive(Debug, Args)]
pub struct LocationGetArgs {
    pub location_id: String,
}

#[derive(Debug, Args)]
pub struct LocationListArgs {
    #[arg(long)]
    pub company: Option<String>,

    #[arg(long, default_value_t = 0)]
    pub skip: u32,

    #[arg(long, default_value_t = 50)]
    pub limit: u32,

    #[arg(long, value_enum, default_value_t = LocationOrder::Asc)]
    pub order: LocationOrder,
}

#[derive(Debug, Args)]
pub struct LocationSearchArgs {
    /// Current GHL API support maps this search value to the upstream email filter.
    pub query: String,

    #[arg(long)]
    pub company: Option<String>,

    #[arg(long, default_value_t = 0)]
    pub skip: u32,

    #[arg(long, default_value_t = 50)]
    pub limit: u32,

    #[arg(long, value_enum, default_value_t = LocationOrder::Asc)]
    pub order: LocationOrder,
}

#[derive(Debug, Subcommand)]
pub enum ContactsCommand {
    List(ContactListArgs),
    Search(ContactSearchArgs),
    Get(ContactGetArgs),
}

#[derive(Debug, Args)]
pub struct ContactListArgs {
    #[arg(long, default_value_t = 25)]
    pub limit: u32,

    #[arg(long)]
    pub start_after_id: Option<String>,

    #[arg(long)]
    pub start_after: Option<u64>,
}

#[derive(Debug, Args)]
pub struct ContactSearchArgs {
    /// Fuzzy contact query. Use --email for exact email filtering.
    pub query: Option<String>,

    #[arg(long)]
    pub email: Option<String>,

    #[arg(long)]
    pub phone: Option<String>,

    #[arg(long, default_value_t = 25)]
    pub limit: u32,

    #[arg(long)]
    pub start_after_id: Option<String>,

    #[arg(long)]
    pub start_after: Option<u64>,
}

#[derive(Debug, Args)]
pub struct ContactGetArgs {
    pub contact_id: String,
}

#[derive(Debug, Subcommand)]
pub enum ConversationsCommand {
    Search(ConversationSearchArgs),
    Get(ConversationGetArgs),
    Messages(ConversationMessagesArgs),
}

#[derive(Debug, Args)]
pub struct ConversationSearchArgs {
    #[arg(long)]
    pub contact: Option<String>,

    #[arg(long)]
    pub query: Option<String>,

    #[arg(long, value_enum, default_value_t = ConversationStatusArg::All)]
    pub status: ConversationStatusArg,

    #[arg(long)]
    pub assigned_to: Option<String>,

    #[arg(long, default_value_t = 20)]
    pub limit: u32,

    #[arg(long)]
    pub last_message_type: Option<String>,

    #[arg(long)]
    pub start_after_date: Option<u64>,
}

#[derive(Debug, Args)]
pub struct ConversationGetArgs {
    pub conversation_id: String,
}

#[derive(Debug, Args)]
pub struct ConversationMessagesArgs {
    pub conversation_id: String,

    #[arg(long, default_value_t = 20)]
    pub limit: u32,

    #[arg(long)]
    pub last_message_id: Option<String>,

    #[arg(long)]
    pub message_type: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum ConversationStatusArg {
    All,
    Read,
    Unread,
    Starred,
    Recents,
}

impl From<ConversationStatusArg> for ghl::ConversationStatus {
    fn from(value: ConversationStatusArg) -> Self {
        match value {
            ConversationStatusArg::All => Self::All,
            ConversationStatusArg::Read => Self::Read,
            ConversationStatusArg::Unread => Self::Unread,
            ConversationStatusArg::Starred => Self::Starred,
            ConversationStatusArg::Recents => Self::Recents,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum PipelinesCommand {
    List,
    Get(PipelineGetArgs),
}

#[derive(Debug, Args)]
pub struct PipelineGetArgs {
    pub pipeline_id: String,
}

#[derive(Debug, Subcommand)]
pub enum OpportunitiesCommand {
    Search(OpportunitySearchArgs),
    Get(OpportunityGetArgs),
}

#[derive(Debug, Args)]
pub struct OpportunitySearchArgs {
    #[arg(long)]
    pub query: Option<String>,

    #[arg(long)]
    pub pipeline: Option<String>,

    #[arg(long)]
    pub stage: Option<String>,

    #[arg(long)]
    pub contact: Option<String>,

    #[arg(long, value_enum)]
    pub status: Option<OpportunityStatusArg>,

    #[arg(long)]
    pub assigned_to: Option<String>,

    #[arg(long, default_value_t = 20)]
    pub limit: u32,

    #[arg(long)]
    pub page: Option<u32>,

    #[arg(long)]
    pub start_after_id: Option<String>,

    #[arg(long)]
    pub start_after: Option<u64>,
}

#[derive(Debug, Args)]
pub struct OpportunityGetArgs {
    pub opportunity_id: String,
}

#[derive(Debug, Subcommand)]
pub enum CalendarsCommand {
    List(CalendarListArgs),
    Get(CalendarGetArgs),
    Events(CalendarEventsArgs),
    FreeSlots(CalendarFreeSlotsArgs),
}

#[derive(Debug, Args)]
pub struct CalendarListArgs {
    #[arg(long)]
    pub group: Option<String>,

    #[arg(long)]
    pub show_drafted: Option<bool>,
}

#[derive(Debug, Args)]
pub struct CalendarGetArgs {
    pub calendar_id: String,
}

#[derive(Debug, Args)]
pub struct CalendarEventsArgs {
    #[arg(long)]
    pub calendar: Option<String>,

    #[arg(long)]
    pub group: Option<String>,

    #[arg(long)]
    pub user: Option<String>,

    /// RFC3339 datetime or epoch milliseconds. Must be paired with --to.
    #[arg(long)]
    pub from: Option<String>,

    /// RFC3339 datetime or epoch milliseconds. Must be paired with --from.
    #[arg(long)]
    pub to: Option<String>,

    /// UTC date range in YYYY-MM-DD format. Mutually exclusive with --from/--to.
    #[arg(long)]
    pub date: Option<String>,
}

#[derive(Debug, Args)]
pub struct CalendarFreeSlotsArgs {
    #[arg(long)]
    pub calendar: String,

    /// UTC date range in YYYY-MM-DD format.
    #[arg(long)]
    pub date: String,

    #[arg(long)]
    pub timezone: Option<String>,

    #[arg(long)]
    pub user: Option<String>,

    #[arg(long, default_value_t = false)]
    pub enable_look_busy: bool,
}

#[derive(Debug, Subcommand)]
pub enum AppointmentsCommand {
    Create(AppointmentCreateArgs),
    Update(AppointmentUpdateArgs),
    Cancel(AppointmentCancelArgs),
    #[command(subcommand)]
    Notes(AppointmentNotesCommand),
}

#[derive(Debug, Subcommand)]
pub enum AppointmentNotesCommand {
    List(AppointmentNotesListArgs),
    Create(AppointmentNoteCreateArgs),
    Update(AppointmentNoteUpdateArgs),
    Delete(AppointmentNoteDeleteArgs),
}

#[derive(Debug, Args)]
pub struct AppointmentNotesListArgs {
    pub appointment_id: String,

    #[arg(long, default_value_t = 10)]
    pub limit: u32,

    #[arg(long, default_value_t = 0)]
    pub offset: u32,
}

#[derive(Debug, Args)]
pub struct AppointmentNoteCreateArgs {
    pub appointment_id: String,

    #[arg(long, conflicts_with = "from_file")]
    pub body: Option<String>,

    #[arg(long = "from-file")]
    pub from_file: Option<PathBuf>,

    #[arg(long)]
    pub user: Option<String>,

    #[arg(long)]
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Args)]
pub struct AppointmentNoteUpdateArgs {
    pub appointment_id: String,
    pub note_id: String,

    #[arg(long, conflicts_with = "from_file")]
    pub body: Option<String>,

    #[arg(long = "from-file")]
    pub from_file: Option<PathBuf>,

    #[arg(long)]
    pub user: Option<String>,

    #[arg(long)]
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Args)]
pub struct AppointmentNoteDeleteArgs {
    pub appointment_id: String,
    pub note_id: String,

    #[arg(long)]
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Args)]
pub struct AppointmentCreateArgs {
    #[arg(long)]
    pub calendar: String,

    #[arg(long)]
    pub contact: String,

    #[arg(long)]
    pub starts_at: String,

    #[arg(long)]
    pub ends_at: String,

    #[arg(long)]
    pub title: Option<String>,

    #[arg(long, value_enum, default_value_t = AppointmentStatusArg::Confirmed)]
    pub status: AppointmentStatusArg,

    #[arg(long)]
    pub assigned_user: Option<String>,

    #[arg(long)]
    pub address: Option<String>,

    #[arg(long)]
    pub meeting_location_type: Option<String>,

    #[arg(long)]
    pub timezone: Option<String>,

    #[arg(long, default_value_t = false)]
    pub ignore_date_range: bool,

    #[arg(long = "no-notify", default_value_t = true, action = clap::ArgAction::SetFalse)]
    pub notify: bool,

    #[arg(long)]
    pub idempotency_key: Option<String>,

    #[arg(long, default_value_t = false)]
    pub skip_free_slot_check: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum AppointmentStatusArg {
    New,
    Confirmed,
}

impl From<AppointmentStatusArg> for ghl::AppointmentStatus {
    fn from(value: AppointmentStatusArg) -> Self {
        match value {
            AppointmentStatusArg::New => Self::New,
            AppointmentStatusArg::Confirmed => Self::Confirmed,
        }
    }
}

#[derive(Debug, Args)]
pub struct AppointmentUpdateArgs {
    pub appointment_id: String,

    #[arg(long)]
    pub title: Option<String>,

    #[arg(long, value_enum)]
    pub status: Option<AppointmentUpdateStatusArg>,

    #[arg(long)]
    pub assigned_user: Option<String>,

    #[arg(long)]
    pub address: Option<String>,

    #[arg(long)]
    pub starts_at: Option<String>,

    #[arg(long)]
    pub ends_at: Option<String>,

    #[arg(long)]
    pub meeting_location_type: Option<String>,

    #[arg(long, default_value_t = false, conflicts_with = "no_notify")]
    pub notify: bool,

    #[arg(long = "no-notify", default_value_t = false)]
    pub no_notify: bool,

    #[arg(long, default_value_t = false)]
    pub ignore_free_slot_validation: bool,

    #[arg(long)]
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Args)]
pub struct AppointmentCancelArgs {
    pub appointment_id: String,

    #[arg(long)]
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum AppointmentUpdateStatusArg {
    New,
    Confirmed,
    Cancelled,
    Showed,
    NoShow,
    Invalid,
}

impl From<AppointmentUpdateStatusArg> for ghl::AppointmentUpdateStatus {
    fn from(value: AppointmentUpdateStatusArg) -> Self {
        match value {
            AppointmentUpdateStatusArg::New => Self::New,
            AppointmentUpdateStatusArg::Confirmed => Self::Confirmed,
            AppointmentUpdateStatusArg::Cancelled => Self::Cancelled,
            AppointmentUpdateStatusArg::Showed => Self::Showed,
            AppointmentUpdateStatusArg::NoShow => Self::NoShow,
            AppointmentUpdateStatusArg::Invalid => Self::Invalid,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum UsersCommand {
    List(UserListArgs),
    Get(UserGetArgs),
    Search(UserSearchArgs),
}

#[derive(Debug, Subcommand)]
pub enum TeamsCommand {
    List(UserListArgs),
}

#[derive(Debug, Args)]
pub struct UserListArgs {
    /// Client-side offset applied after fetching the location user list.
    #[arg(long, default_value_t = 0)]
    pub skip: u32,

    /// Client-side maximum number of user IDs to print.
    #[arg(long, default_value_t = 25)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct UserGetArgs {
    pub user_id: String,
}

#[derive(Debug, Args)]
pub struct UserSearchArgs {
    /// Company-wide user search query. Requires company context.
    #[arg(long)]
    pub query: Option<String>,

    /// Location-scoped exact email lookup. Requires location context.
    #[arg(long)]
    pub email: Option<String>,

    #[arg(long, default_value_t = 0)]
    pub skip: u32,

    #[arg(long, default_value_t = 25)]
    pub limit: u32,
}

#[derive(Debug, Subcommand)]
pub enum SmokeCommand {
    Run(SmokeRunArgs),
}

#[derive(Debug, Args)]
pub struct SmokeRunArgs {
    #[arg(long, default_value_t = 1)]
    pub limit: u32,

    #[arg(long, default_value_t = false)]
    pub skip_optional: bool,

    #[arg(long)]
    pub contact_query: Option<String>,

    #[arg(long)]
    pub contact_email: Option<String>,

    #[arg(long)]
    pub contact_phone: Option<String>,

    #[arg(long)]
    pub contact_id: Option<String>,

    #[arg(long)]
    pub conversation_id: Option<String>,

    #[arg(long)]
    pub pipeline_id: Option<String>,

    #[arg(long)]
    pub opportunity_id: Option<String>,

    #[arg(long)]
    pub calendar_id: Option<String>,

    #[arg(long)]
    pub calendar_date: Option<String>,

    #[arg(long)]
    pub calendar_timezone: Option<String>,

    #[arg(long)]
    pub user_id: Option<String>,

    #[arg(long)]
    pub user_email: Option<String>,

    #[arg(long)]
    pub user_query: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum OpportunityStatusArg {
    Open,
    Won,
    Lost,
    Abandoned,
    All,
}

impl From<OpportunityStatusArg> for ghl::OpportunityStatus {
    fn from(value: OpportunityStatusArg) -> Self {
        match value {
            OpportunityStatusArg::Open => Self::Open,
            OpportunityStatusArg::Won => Self::Won,
            OpportunityStatusArg::Lost => Self::Lost,
            OpportunityStatusArg::Abandoned => Self::Abandoned,
            OpportunityStatusArg::All => Self::All,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum LocationOrder {
    Asc,
    Desc,
}

impl From<LocationOrder> for ghl::LocationSearchOrder {
    fn from(value: LocationOrder) -> Self {
        match value {
            LocationOrder::Asc => Self::Asc,
            LocationOrder::Desc => Self::Desc,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum RawSurface {
    Services,
    Backend,
}

impl From<RawSurface> for ghl::Surface {
    fn from(value: RawSurface) -> Self {
        match value {
            RawSurface::Services => Self::Services,
            RawSurface::Backend => Self::Backend,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum RawMethod {
    Get,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum RawAuthClass {
    Pit,
}

impl From<RawAuthClass> for ghl::AuthClass {
    fn from(value: RawAuthClass) -> Self {
        match value {
            RawAuthClass::Pit => Self::Pit,
        }
    }
}

#[derive(Debug, Args)]
pub struct EndpointShowArgs {
    pub endpoint_key: String,
}

#[derive(Debug, Args)]
pub struct CompletionsArgs {
    pub shell: CompletionShell,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    Powershell,
}

impl From<CompletionShell> for clap_complete::Shell {
    fn from(value: CompletionShell) -> Self {
        match value {
            CompletionShell::Bash => Self::Bash,
            CompletionShell::Zsh => Self::Zsh,
            CompletionShell::Fish => Self::Fish,
            CompletionShell::Powershell => Self::PowerShell,
        }
    }
}
