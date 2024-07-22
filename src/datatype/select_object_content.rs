use std::fmt::Display;

use super::ToXml;

/// `select_object_content` method parameters.
#[derive(Clone)]
pub struct SelectRequest {
    expression: String,
    input_serialization: InputSerialization,
    pub(crate) output_serialization: OutputSerialization,
    request_progress: bool,
    scan_start_range: Option<usize>,
    scan_end_range: Option<usize>,
}

impl SelectRequest {
    /// Params:
    /// - expression: The expression that is used to query the object.\
    /// - input_serialization: [InputSerialization] Describes the format of the data in the object that is being queried.
    /// - output_serialization: [OutputSerialization] Describes the format of the data that you want Amazon S3 to return in response.
    /// - request_progress: Specifies if periodic request progress information should be enabled.
    /// - scan_start_range: *Optional*, Specifies the start of the byte range.
    /// - scan_end_range: *Optional*, Specifies the end of the byte range.
    pub fn new(
        expression: String,
        input_serialization: InputSerialization,
        output_serialization: OutputSerialization,
        request_progress: bool,
        scan_start_range: Option<usize>,
        scan_end_range: Option<usize>,
    ) -> Self {
        Self {
            expression: expression
                .replace("&", "&amp;")
                .replace("<", "&lt;")
                .replace(">", "&gt;"),
            // .replace("\"", "&quot;")
            // .replace("'", "&apos;"),
            input_serialization,
            output_serialization,
            request_progress,
            scan_start_range,
            scan_end_range,
        }
    }
}

impl ToXml for SelectRequest {
    fn to_xml(&self) -> crate::error::Result<String> {
        let expression = &self.expression;
        let progress = self.request_progress;
        let input = &self.input_serialization;
        let output = &self.output_serialization;
        let start = if let Some(start) = self.scan_start_range {
            format!("<start>{start}</start>")
        } else {
            "".to_string()
        };
        let end = if let Some(end) = self.scan_end_range {
            format!("<end>{end}</end>")
        } else {
            "".to_string()
        };
        Ok(format!("<SelectObjectContentRequest><Expression>{expression}</Expression><ExpressionType>SQL</ExpressionType>{input}{output}<RequestProgress><Enabled>{progress}</Enabled></RequestProgress><scanrange>{start}{end}</scanrange></SelectObjectContentRequest>"))
    }
}

/// Specifies object's compression format,
/// Valid values: `NONE`, `GZIP`, `BZIP2`. Default Value: `NONE`.
#[derive(Debug, Default, Clone, Copy)]
pub enum CompressionType {
    #[default]
    NONE,
    GZIP,
    BZIP2,
}

impl CompressionType {
    pub fn as_str(&self) -> &str {
        match self {
            CompressionType::NONE => "NONE",
            CompressionType::GZIP => "GZIP",
            CompressionType::BZIP2 => "BZIP2",
        }
    }
}

impl Display for CompressionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Describes the first line of input.
/// Valid Values: `USE` | `IGNORE` | `NONE`.
/// Default: `IGNORE`
#[derive(Debug, Default, Clone, Copy)]
pub enum FileHeaderInfo {
    /// First line is not a header.
    NONE,
    /// First line is a header, but you can't use the header values to indicate the column in an expression.
    /// You can use column position (such as _1, _2, …) to indicate the column (`SELECT s._1 FROM OBJECT s`).
    #[default]
    IGNORE,
    /// First line is a header, and you can use the header value to identify a column in an expression (`SELECT "name" FROM OBJECT`).
    USE,
}

impl Display for FileHeaderInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileHeaderInfo::NONE => f.write_str("NONE"),
            FileHeaderInfo::IGNORE => f.write_str("IGNORE"),
            FileHeaderInfo::USE => f.write_str("USE"),
        }
    }
}

/// Describes how an uncompressed comma-separated values (CSV)-formatted input object is formatted.
#[derive(Debug, Clone, Copy)]
pub struct CsvInput {
    /// Specifies that CSV field values may contain quoted record delimiters and such records should be allowed.
    /// Default value is FALSE. Setting this value to TRUE may lower performance.
    allow_quoted_record_delimiter: bool,

    /// A single character used to indicate that a row should be ignored when the character is present at the start of that row.
    /// You can specify any character to indicate a comment line. The default character is #.
    comments: char,
    /// A single character used to separate individual fields in a record. You can specify an arbitrary delimiter.
    /// The default character is ','.
    field_delimiter: char,
    /// Describes the first line of input. Valid values are: USE | IGNORE | NONE
    file_header_info: FileHeaderInfo,
    /// A single character used for escaping when the field delimiter is part of the value.
    /// For example, if the value is a, b, Amazon S3 wraps this field value in quotation marks, as follows: " a , b ".
    /// The default character is `"`.
    quote_character: char,
    /// A single character used for escaping the quotation mark character inside an already escaped value.
    /// For example, the value """ a , b """ is parsed as " a , b ".
    /// The default character is `"`.
    quote_escape_character: char,
    /// A single character used to separate individual records in the input.
    /// The default character is `\n`.
    record_delimiter: char,
}

impl CsvInput {
    pub fn new(
        allow_quoted_record_delimiter: bool,
        comments: char,
        field_delimiter: char,
        file_header_info: FileHeaderInfo,
        quote_character: char,
        quote_escape_character: char,
        record_delimiter: char,
    ) -> Self {
        Self {
            allow_quoted_record_delimiter,
            comments,
            field_delimiter,
            file_header_info,
            quote_character,
            quote_escape_character,
            record_delimiter,
        }
    }
}

impl Default for CsvInput {
    /// Default CsvInput
    /// - allow_quoted_record_delimiter: `false`
    /// - comments: `#`
    /// - field_delimiter： `,`
    /// - file_header_info [FileHeaderInfo::IGNORE]
    /// - quote_character `"`
    /// - quote_escape_character `"`
    /// - record_delimiter `\n`
    fn default() -> Self {
        Self::new(false, '#', ',', Default::default(), '"', '"', '\n')
    }
}

impl Display for CsvInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"<CSV><FileHeaderInfo>{}</FileHeaderInfo><RecordDelimiter>{}</RecordDelimiter><FieldDelimiter>{}</FieldDelimiter><QuoteCharacter>{}</QuoteCharacter><QuoteEscapeCharacter>{}</QuoteEscapeCharacter><Comments>{}</Comments><AllowQuotedRecordDelimiter>{}</AllowQuotedRecordDelimiter></CSV>",
        self.file_header_info,self.record_delimiter,self.field_delimiter,self.quote_character,self.quote_escape_character,self.comments,self.allow_quoted_record_delimiter
    )
    }
}

/// Specifies JSON as object's input serialization format.
#[derive(Debug, Default, Clone, Copy)]
pub struct JsonInput {
    /// The type of JSON. Valid values: Document, Lines.
    /// true => Document
    /// false => Lines
    json_type: bool,
}

impl JsonInput {
    /// Document Json type.
    ///
    /// Indicates that the JSON file contains only one JSON object, and that the object can be sliced into multiple lines.
    pub fn document() -> Self {
        Self { json_type: true }
    }
    /// Lines Json type.
    ///
    /// Indicates that each row contains a separate JSON object.
    pub fn lines() -> Self {
        Self { json_type: false }
    }
}

impl Display for JsonInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ty = if self.json_type { "Document" } else { "Lines" };
        write!(f, "<JSON><Type>{ty}</Type></JSON>")
    }
}

/// Container for Parquet.
#[derive(Debug, Default, Clone, Copy)]
pub struct ParquetInput;

impl Display for ParquetInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<Parquet></Parquet>")
    }
}

/// Input serialization
#[derive(Debug, Clone, Copy)]
pub enum Input {
    Csv(CsvInput),
    Json(JsonInput),
    Parquet(ParquetInput),
}

impl From<CsvInput> for Input {
    fn from(value: CsvInput) -> Self {
        Self::Csv(value)
    }
}

impl From<JsonInput> for Input {
    fn from(value: JsonInput) -> Self {
        Self::Json(value)
    }
}

impl From<ParquetInput> for Input {
    fn from(value: ParquetInput) -> Self {
        Self::Parquet(value)
    }
}

impl Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Input::Csv(i) => write!(f, "{i}"),
            Input::Json(i) => write!(f, "{i}"),
            Input::Parquet(i) => write!(f, "{i}"),
        }
    }
}

/// Describes the serialization format of the object.
#[derive(Clone, Copy)]
pub struct InputSerialization {
    compression_type: CompressionType,
    input: Input,
}

impl InputSerialization {
    pub fn new<I: Into<Input>>(input: I, compression_type: CompressionType) -> Self {
        Self {
            compression_type,
            input: input.into(),
        }
    }
}

impl Display for InputSerialization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<InputSerialization><CompressionType>{}</CompressionType>{}</InputSerialization>",
            self.compression_type, &self.input
        )
    }
}

/// Indicates whether to use quotation marks around output fields.
#[derive(Debug, Clone, Copy, Default)]
pub enum QuoteFields {
    /// Always use quotation marks for output fields.
    ALWAYS,
    #[default]
    /// Use quotation marks for output fields when needed.
    ASNEEDED,
}

impl Display for QuoteFields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuoteFields::ALWAYS => f.write_str("ALWAYS"),
            QuoteFields::ASNEEDED => f.write_str("ASNEEDED"),
        }
    }
}

/// Describes the serialization of CSV-encoded Select results.
#[derive(Clone)]
pub struct CsvOutput {
    field_delimiter: char,
    quote_character: char,
    quote_escape_character: char,
    quote_fields: QuoteFields,
    record_delimiter: String,
}

impl CsvOutput {
    pub fn new(
        field_delimiter: char,
        quote_character: char,
        quote_escape_character: char,
        quote_fields: QuoteFields,
        record_delimiter: String,
    ) -> Self {
        CsvOutput {
            field_delimiter,
            quote_character,
            quote_escape_character,
            quote_fields,
            record_delimiter,
        }
    }

    pub fn record_delimiter(&self) -> &str {
        self.record_delimiter.as_str()
    }
}

impl Default for CsvOutput {
    /// Default CsvOutput
    /// - field_delimiter: `,`
    /// - quote_character: `"`,
    /// - quote_escape_character: `"`,
    /// - quote_fields: [QuoteFields::ASNEEDED],
    /// - record_delimiter: `\n`,
    fn default() -> Self {
        Self::new(',', '"', '"', Default::default(), "\n".to_owned())
    }
}

impl Display for CsvOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"<CSV><FieldDelimiter>{}</FieldDelimiter><QuoteCharacter>{}</QuoteCharacter><QuoteEscapeCharacter>{}</QuoteEscapeCharacter><QuoteFields>{}</QuoteFields><RecordDelimiter>{}</RecordDelimiter></CSV>",
        self.field_delimiter,self.quote_character,self.quote_escape_character,self.quote_fields,self.record_delimiter)
    }
}

/// Specifies JSON as request's output serialization format.
#[derive(Clone)]
pub struct JsonOutput(String);

impl JsonOutput {
    /// record_delimiter: used to separate individual records in the output.
    pub fn new<S: Into<String>>(record_delimiter: S) -> Self {
        Self(record_delimiter.into())
    }

    pub fn record_delimiter(&self) -> &str {
        self.0.as_str()
    }
}

impl Default for JsonOutput {
    /// Default JsonOutput with record_delimiter `\n`
    fn default() -> Self {
        Self::new("\n")
    }
}

impl Display for JsonOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<JSON><RecordDelimiter>{}</RecordDelimiter></JSON>",
            self.0
        )
    }
}

/// Describes the format of the data that you want Amazon S3 to return in response.
#[derive(Clone)]
pub enum OutputSerialization {
    Csv(CsvOutput),
    Json(JsonOutput),
}

impl OutputSerialization {
    pub fn record_delimiter(&self) -> &str {
        match self {
            OutputSerialization::Csv(csv) => csv.record_delimiter(),
            OutputSerialization::Json(js) => js.record_delimiter(),
        }
    }
}

impl Display for OutputSerialization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputSerialization::Csv(i) => {
                write!(f, "<OutputSerialization>{}</OutputSerialization>", i)
            }
            OutputSerialization::Json(i) => {
                write!(f, "<OutputSerialization>{}</OutputSerialization>", i)
            }
        }
    }
}

impl From<CsvOutput> for OutputSerialization {
    fn from(value: CsvOutput) -> Self {
        Self::Csv(value)
    }
}

impl From<JsonOutput> for OutputSerialization {
    fn from(value: JsonOutput) -> Self {
        Self::Json(value)
    }
}
