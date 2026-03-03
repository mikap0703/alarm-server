# alarm-server

## Configuration Overview

The app loads three JSON files from `config/` at startup:

- `config/general.json`
- `config/alarm_sources.json`
- `config/alarm_templates.json`

If a file is missing or invalid JSON, startup fails.

## `config/general.json`

Top-level fields:

- `apis` (array, required): API backends that can receive alarms.
- `timeout` (u64 seconds, required): time window used to classify new alarms as updates vs first alarms.
- `source_priority` (array of strings, required): source ranking used when multiple sources produce alarms inside `timeout`.
- `alarm` (bool, required): global outbound dispatch switch.

`alarm` behavior:

- `true`: API dispatch and webhook calls are enabled.
- `false`: API dispatch and webhook calls are disabled.
- Alarm ingestion/parsing from mail/serial still runs.

### `apis` entries

Each item in `apis` has:

- `name` (string, required): logical API name. Must match template target names in `alarm_templates.json` (for example `Divera`, `Telegram`).
- `api` (enum string, required): one of:
- `Divera`: sends alarms to Divera 24/7 API v2.
- `Telegram`: sends Telegram messages.
- `Alamos`: currently mapped to a mock/no-op implementation.
- `api_key` (string, required): credential/token for the selected `api` type.

API key meaning by type:

- `Divera`: Divera `accesskey`.
- `Telegram`: bot token (format like `123456:ABC...`).
- `Alamos`: currently unused by mock backend.

## `config/alarm_sources.json`

Top-level fields:

- `mail_sources` (array, required)
- `serial_sources` (array, required)

### `mail_sources` entries

Fields:

- `name` (string, required): source ID. Used as alarm origin and for `source_priority` matching.
- `active` (bool, required): if `false`, source is skipped at startup.
- `user` (string, required): IMAP username.
- `password` (string, required): IMAP password.
- `host` (string, required): IMAP host.
- `port` (u16, required): IMAP port.
- `tls` (bool, required in schema): currently not used by runtime logic.
- `max_age` (u64 seconds, required): reject mails older than this value. `0` disables age filtering.
- `alarm_sender` (string, required): expected sender address. Use `*` as wildcard.
- `alarm_subject` (string, required): expected subject. Use `*` as wildcard.
- `alarm_template_keywords` (map string->string, required): maps detected unit names to template names.
- `mail_schema` (string, required): parser selection.
- `stichwoerter` (map string->string, required): keyword normalization map used by SecurCAD parser.
- `ignore_units` (array of strings, required): units to exclude from parsed unit list.
- `polling` (bool, required): enable polling loop.
- `polling_interval` (u64 seconds, required): polling interval.
- `idle` (bool, required): enable IMAP IDLE loop.

`mail_schema` options:

- `SL-securCAD`: structured parser for secur.CAD mails.
- `Plaintext`: plain text parser (appends text body to alarm text).
- any other string: falls back to mock parser.

### `serial_sources` entries

Fields:

- `name` (string, required): source ID. Used as alarm origin and for `source_priority` matching.
- `active` (bool, required): if `false`, source is skipped at startup.
- `port` (string, required): serial device path.
- `delimiter` (string, required): message delimiter. Escapes like `\\r`, `\\n`, `\\0` are supported.
- `baudrate` (u32, required): serial baud rate.
- `alarm_list` (array of strings, required): if message text contains one of these values, it is used as title.
- `rics` (map string->string, required): maps RIC codes to template names.

## `config/alarm_templates.json`

Top-level object keys are template names (for example `default`, `Neuweiler Vollalarm`).

Each template contains dynamic targets:

- API target key (for example `Divera`, `Telegram`), with object value:
- `members` (optional string array)
- `groups` (optional string array)
- `vehicles` (optional string array)
- `Webhooks`, with array value of URL strings.

Behavior:

- `default` template is always applied first to every alarm.
- Additional template names from parsers/sources are applied afterwards.
- API target keys must match `general.apis[].name` to dispatch.
- `Webhooks` entries are executed as HTTP GET calls when `general.alarm` is `true`.

## Minimal Example

```json
{
  "apis": [
    { "name": "Divera", "api": "Divera", "api_key": "your-divera-accesskey" },
    { "name": "Telegram", "api": "Telegram", "api_key": "your-telegram-bot-token" }
  ],
  "source_priority": ["Inbox", "Serial"],
  "timeout": 5000,
  "alarm": true
}
```
