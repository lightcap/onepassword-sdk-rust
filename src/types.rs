use serde::{Deserialize, Serialize};

pub type ErrorMessage = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressFieldDetails {
    pub street: String,
    pub city: String,
    pub country: String,
    pub zip: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentCreateParams {
    pub name: String,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttributes {
    pub name: String,
    pub id: String,
    pub size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileCreateParams {
    pub name: String,
    pub content: Vec<u8>,
    pub section_id: String,
    pub field_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentVariable {
    pub name: String,
    pub value: String,
    pub masked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetVariablesResponse {
    pub variables: Vec<EnvironmentVariable>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GeneratePasswordResponse {
    pub password: String,
}

impl std::fmt::Debug for GeneratePasswordResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeneratePasswordResponse")
            .field("password", &"[REDACTED]")
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SeparatorType {
    Digits,
    DigitsAndSymbols,
    Spaces,
    Hyphens,
    Underscores,
    Periods,
    Commas,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WordListType {
    FullWords,
    Syllables,
    ThreeLetters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordRecipeMemorableInner {
    pub separator_type: SeparatorType,
    pub capitalize: bool,
    pub word_list_type: WordListType,
    pub word_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordRecipePinInner {
    pub length: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordRecipeRandomInner {
    pub include_digits: bool,
    pub include_symbols: bool,
    pub length: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "parameters")]
pub enum PasswordRecipe {
    Memorable(PasswordRecipeMemorableInner),
    Pin(PasswordRecipePinInner),
    Random(PasswordRecipeRandomInner),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SSHKeyAttributes {
    pub public_key: String,
    pub fingerprint: String,
    pub key_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OTPFieldDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum ItemFieldDetails {
    Otp(OTPFieldDetails),
    SshKey(SSHKeyAttributes),
    Address(AddressFieldDetails),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemCategory {
    Login,
    SecureNote,
    CreditCard,
    CryptoWallet,
    Identity,
    Password,
    Document,
    ApiCredentials,
    BankAccount,
    Database,
    DriverLicense,
    Email,
    MedicalRecord,
    Membership,
    OutdoorLicense,
    Passport,
    Rewards,
    Router,
    Server,
    SshKey,
    SocialSecurityNumber,
    SoftwareLicense,
    Person,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemFieldType {
    Text,
    Concealed,
    CreditCardType,
    CreditCardNumber,
    Phone,
    Url,
    Totp,
    Email,
    Reference,
    SshKey,
    Menu,
    MonthYear,
    Address,
    Date,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemState {
    Active,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutofillBehavior {
    AnywhereOnWebsite,
    ExactDomain,
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Website {
    pub url: String,
    pub label: String,
    pub autofill_behavior: AutofillBehavior,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemField {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_id: Option<String>,
    pub field_type: ItemFieldType,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<ItemFieldDetails>,
}

impl std::fmt::Debug for ItemField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ItemField")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("section_id", &self.section_id)
            .field("field_type", &self.field_type)
            .field("value", &"[REDACTED]")
            .field("details", &self.details)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSection {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemFile {
    pub attributes: FileAttributes,
    pub section_id: String,
    pub field_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: String,
    pub title: String,
    pub category: ItemCategory,
    pub vault_id: String,
    pub fields: Vec<ItemField>,
    pub sections: Vec<ItemSection>,
    pub notes: String,
    pub tags: Vec<String>,
    pub websites: Vec<Website>,
    pub version: u32,
    pub files: Vec<ItemFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<FileAttributes>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemCreateParams {
    pub category: ItemCategory,
    pub vault_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<ItemField>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sections: Option<Vec<ItemSection>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub websites: Option<Vec<Website>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<FileCreateParams>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<DocumentCreateParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemOverview {
    pub id: String,
    pub title: String,
    pub category: ItemCategory,
    pub vault_id: String,
    pub websites: Vec<Website>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub state: ItemState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemListFilterByStateInner {
    pub active: bool,
    pub archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum ItemListFilter {
    ByState(ItemListFilterByStateInner),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemShareDuration {
    OneHour,
    OneDay,
    SevenDays,
    FourteenDays,
    ThirtyDays,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllowedType {
    Authenticated,
    Public,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllowedRecipientType {
    Email,
    Domain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemShareFiles {
    pub allowed: bool,
    pub max_size: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_types: Option<Vec<AllowedType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_recipient_types: Option<Vec<AllowedRecipientType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_expiry: Option<ItemShareDuration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_expiry: Option<ItemShareDuration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_views: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemShareAccountPolicy {
    pub max_expiry: ItemShareDuration,
    pub default_expiry: ItemShareDuration,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_views: Option<u32>,
    pub allowed_types: Vec<AllowedType>,
    pub allowed_recipient_types: Vec<AllowedRecipientType>,
    pub files: ItemShareFiles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidRecipientEmailInner {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidRecipientDomainInner {
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "parameters")]
pub enum ValidRecipient {
    Email(ValidRecipientEmailInner),
    Domain(ValidRecipientDomainInner),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemShareParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipients: Option<Vec<ValidRecipient>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_after: Option<ItemShareDuration>,
    pub one_time_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse<T, E> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<E>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum ItemUpdateFailureReason {
    ItemValidationError(ErrorMessage),
    ItemStatusPermissionError,
    ItemStatusIncorrectItemVersion,
    ItemStatusFileNotFound,
    ItemStatusTooBig,
    ItemNotFound,
    Internal(ErrorMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemsDeleteAllResponse {
    pub individual_responses:
        std::collections::HashMap<String, BatchResponse<(), ItemUpdateFailureReason>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemsGetAllError {
    #[serde(rename = "itemNotFound")]
    ItemNotFound,
    #[serde(rename = "internal")]
    Internal(ErrorMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemsGetAllResponse {
    pub individual_responses: Vec<BatchResponse<Item, ItemsGetAllError>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemsUpdateAllResponse {
    pub individual_responses: Vec<BatchResponse<Item, ItemUpdateFailureReason>>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedReference {
    pub secret: String,
    pub item_id: String,
    pub vault_id: String,
}

impl std::fmt::Debug for ResolvedReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResolvedReference")
            .field("secret", &"[REDACTED]")
            .field("item_id", &self.item_id)
            .field("vault_id", &self.vault_id)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum ResolveReferenceError {
    Parsing(ErrorMessage),
    FieldNotFound,
    VaultNotFound,
    TooManyVaults,
    ItemNotFound,
    TooManyItems,
    TooManyMatchingFields,
    NoMatchingSections,
    IncompatibleTOTPQueryParameterField,
    #[serde(rename = "unableToGenerateTotpCode")]
    UnableToGenerateTOTPCode(ErrorMessage),
    #[serde(rename = "sSHKeyMetadataNotFound")]
    SSHKeyMetadataNotFound,
    UnsupportedFileFormat,
    #[serde(rename = "incompatibleSshKeyQueryParameterField")]
    IncompatibleSSHKeyQueryParameterField,
    UnableToParsePrivateKey,
    #[serde(rename = "unableToFormatPrivateKeyToOpenSsh")]
    UnableToFormatPrivateKeyToOpenSSH,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveAllResponse {
    pub individual_responses:
        std::collections::HashMap<String, BatchResponse<ResolvedReference, ResolveReferenceError>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GroupType {
    Owners,
    Administrators,
    Recovery,
    ExternalAccountManagers,
    TeamMembers,
    UserDefined,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GroupState {
    Active,
    Deleted,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VaultAccessorType {
    User,
    Group,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultAccess {
    pub vault_uuid: String,
    pub accessor_type: VaultAccessorType,
    pub accessor_uuid: String,
    pub permissions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: String,
    pub title: String,
    pub description: String,
    pub group_type: GroupType,
    pub state: GroupState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_access: Option<Vec<VaultAccess>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupAccess {
    pub group_id: String,
    pub permissions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupGetParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_permissions: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupVaultAccess {
    pub vault_id: String,
    pub group_id: String,
    pub permissions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VaultType {
    Personal,
    Everyone,
    Transfer,
    UserCreated,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vault {
    pub id: String,
    pub title: String,
    pub description: String,
    pub vault_type: VaultType,
    pub active_item_count: u32,
    pub content_version: u32,
    pub attribute_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access: Option<Vec<VaultAccess>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultCreateParams {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_admins_access: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultGetParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessors: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decrypt_details: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultOverview {
    pub id: String,
    pub title: String,
    pub description: String,
    pub vault_type: VaultType,
    pub active_item_count: u32,
    pub content_version: u32,
    pub attribute_version: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultUpdateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

pub mod permissions {
    pub const READ_ITEMS: u32 = 32;
    pub const REVEAL_ITEM_PASSWORD: u32 = 16;
    pub const UPDATE_ITEMS: u32 = 64;
    pub const CREATE_ITEMS: u32 = 128;
    pub const ARCHIVE_ITEMS: u32 = 256;
    pub const DELETE_ITEMS: u32 = 512;
    pub const UPDATE_ITEM_HISTORY: u32 = 1024;
    pub const SEND_ITEMS: u32 = 1_048_576;
    pub const IMPORT_ITEMS: u32 = 2_097_152;
    pub const EXPORT_ITEMS: u32 = 4_194_304;
    pub const PRINT_ITEMS: u32 = 8_388_608;
    pub const MANAGE_VAULT: u32 = 2;
    pub const RECOVER_VAULT: u32 = 1;
    pub const NO_ACCESS: u32 = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_recipe_serializes_as_tagged_enum() {
        let recipe = PasswordRecipe::Random(PasswordRecipeRandomInner {
            include_digits: true,
            include_symbols: false,
            length: 20,
        });
        let json = serde_json::to_string(&recipe).unwrap();
        assert!(json.contains("\"type\":\"Random\""));
        assert!(json.contains("\"parameters\":"));
    }

    #[test]
    fn file_create_params_serializes_content_as_byte_array() {
        let params = FileCreateParams {
            name: "file.txt".to_string(),
            content: b"hello".to_vec(),
            section_id: "section".to_string(),
            field_id: "field".to_string(),
        };

        let value = serde_json::to_value(&params).unwrap();
        assert_eq!(
            value["content"],
            serde_json::json!([104, 101, 108, 108, 111])
        );

        let parsed: FileCreateParams = serde_json::from_value(value).unwrap();
        assert_eq!(parsed.content, b"hello");
    }

    #[test]
    fn document_create_params_serializes_content_as_byte_array() {
        let params = DocumentCreateParams {
            name: "document.txt".to_string(),
            content: b"document".to_vec(),
        };

        let value = serde_json::to_value(&params).unwrap();
        assert_eq!(
            value["content"],
            serde_json::json!([100, 111, 99, 117, 109, 101, 110, 116])
        );

        let parsed: DocumentCreateParams = serde_json::from_value(value).unwrap();
        assert_eq!(parsed.content, b"document");
    }

    #[test]
    fn file_read_response_deserializes_from_byte_array() {
        let parsed: Vec<u8> = serde_json::from_str("[114,101,115,112,111,110,115,101]").unwrap();
        assert_eq!(parsed, b"response");
    }

    #[test]
    fn item_field_details_roundtrip() {
        let details = ItemFieldDetails::Otp(OTPFieldDetails {
            code: Some("123456".to_string()),
            error_message: None,
        });
        let json = serde_json::to_string(&details).unwrap();
        let parsed: ItemFieldDetails = serde_json::from_str(&json).unwrap();
        match parsed {
            ItemFieldDetails::Otp(otp) => assert_eq!(otp.code.unwrap(), "123456"),
            _ => panic!("expected Otp variant"),
        }
    }

    #[test]
    fn valid_recipient_roundtrip() {
        let recipient = ValidRecipient::Email(ValidRecipientEmailInner {
            email: "test@example.com".to_string(),
        });
        let json = serde_json::to_string(&recipient).unwrap();
        assert!(json.contains("\"type\":\"Email\""));
        let parsed: ValidRecipient = serde_json::from_str(&json).unwrap();
        match parsed {
            ValidRecipient::Email(inner) => assert_eq!(inner.email, "test@example.com"),
            _ => panic!("expected Email variant"),
        }
    }

    #[test]
    fn item_list_filter_roundtrip() {
        let filter = ItemListFilter::ByState(ItemListFilterByStateInner {
            active: true,
            archived: false,
        });
        let json = serde_json::to_string(&filter).unwrap();
        let parsed: ItemListFilter = serde_json::from_str(&json).unwrap();
        match parsed {
            ItemListFilter::ByState(inner) => {
                assert!(inner.active);
                assert!(!inner.archived);
            }
        }
    }
}
