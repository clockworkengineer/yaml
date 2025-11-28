
# YAML to Bencode Converter Example

This example demonstrates how to convert YAML files to Bencode format using the YAML library's Bencode serialization capabilities. Error handling and validation are supported for robust conversion.


## What This Example Shows

- Reading YAML files using `FileSource`
- Parsing YAML content into Node structures
- Converting Node trees to Bencode format with `to_bencode()`
- Writing Bencode output to files using `FileDestination`
- Batch processing multiple files
- Error codes and recovery for malformed files

## What is Bencode?

Bencode is the encoding used by the BitTorrent peer-to-peer file sharing system for storing and transmitting loosely structured data. It's simple, unambiguous, and easy to parse.

**Bencode supports four data types:**
- **Byte strings** - `4:spam` (length prefix followed by data)
- **Integers** - `i42e` (integer between 'i' and 'e')
- **Lists** - `l4:spam4:eggse` (list between 'l' and 'e')
- **Dictionaries** - `d3:cow3:moo4:spam4:eggse` (dictionary between 'd' and 'e')

## How It Works

The example:
1. Scans the `files/` directory for all `.yaml` files
2. For each YAML file:
   - Opens and parses the YAML content
   - Converts the parsed Node tree to Bencode format
   - Writes the Bencode output to a new file with `.bencode` extension


This is useful for:
- **BitTorrent applications** - Creating or manipulating .torrent files
- **P2P systems** - Data exchange in peer-to-peer networks
- **Compact serialization** - Efficient binary data representation
- **Cross-format conversion** - Moving data between YAML and Bencode
- **Error diagnostics** - Get detailed error codes and suggestions for invalid files

## Usage

```bash
# Run the example from the project root
cargo run --example yaml_to_bencode

# Or run from the examples directory
cd examples/yaml_to_bencode
cargo run
```

## Example Input/Output

### Input (`torrent-info.yaml`)
```yaml
announce: http://tracker.example.com:8080/announce
info:
  name: example-file.txt
  piece_length: 16384
  length: 1048576
  pieces: "binary_hash_data_here"
created_by: YAML_lib
creation_date: 1699564800
```

### Output (`torrent-info.bencode`)
```
d8:announce38:http://tracker.example.com:8080/announce4:infod6:lengthi1048576e4:name16:example-file.txt12:piece_lengthi16384e6:pieces21:binary_hash_data_heree10:created_by8:YAML_lib13:creation_datei1699564800ee
```

### Decoded Bencode Structure
```
dictionary {
  "announce": "http://tracker.example.com:8080/announce"
  "info": dictionary {
    "length": 1048576
    "name": "example-file.txt"
    "piece_length": 16384
    "pieces": "binary_hash_data_here"
  }
  "created_by": "YAML_lib"
  "creation_date": 1699564800
}
```

## Key Functions Used

- **`FileSource::new(path)`** - Opens a YAML file for reading
- **`parse(&mut source)`** - Parses YAML into a Node tree
- **`FileDestination::new(path)`** - Creates a destination file for writing
- **`to_bencode(&node, &mut destination)`** - Converts Node tree to Bencode format

## YAML to Bencode Mapping

The conversion handles:
- **Mappings** → Bencode dictionaries (d...e)
- **Sequences** → Bencode lists (l...e)
- **Strings** → Bencode byte strings (length:data)
- **Numbers (integers)** → Bencode integers (i...e)
- **Numbers (floats)** → Converted to strings then bencoded
- **Booleans** → Converted to integers (0 or 1)
- **Null** → Omitted (Bencode doesn't support null)

## Bencode Format Details

### String Encoding
```
4:spam  -> "spam"
0:      -> "" (empty string)
11:hello world -> "hello world"
```

### Integer Encoding
```
i42e    -> 42
i-42e   -> -42
i0e     -> 0
```

### List Encoding
```
l4:spam4:eggse  -> ["spam", "eggs"]
li1ei2ei3ee     -> [1, 2, 3]
le              -> [] (empty list)
```

### Dictionary Encoding
```
d3:cow3:moo4:spam4:eggse  -> {"cow": "moo", "spam": "eggs"}
de                         -> {} (empty dictionary)
```

**Note:** Dictionary keys must be strings and are sorted lexicographically in Bencode.

## BitTorrent Use Case

Bencode is primarily used for `.torrent` files. A typical torrent file structure in YAML:

```yaml
announce: http://tracker.example.com/announce
announce-list:
  - - http://tracker1.example.com/announce
  - - http://tracker2.example.com/announce
info:
  name: MyFile.zip
  piece_length: 262144
  pieces: <binary SHA1 hashes>
  length: 10485760
```

After conversion to Bencode, this becomes a valid .torrent file.

## Error Handling

The example demonstrates error handling for:
- Missing or unreadable files
- Invalid YAML syntax
- Incompatible data types for Bencode
- File write permissions
- Each file is processed independently

## Bencode Limitations

When converting YAML to Bencode, be aware:
- **No null values** - Bencode doesn't support null
- **String keys only** - Dictionary keys must be strings
- **No floats** - Floating-point numbers are converted to strings
- **Binary data** - All strings are byte strings (not UTF-8 guaranteed)
- **Sorted keys** - Dictionary keys are automatically sorted

## See Also

- **yaml_to_json** - Converting YAML to JSON format
- **yaml_to_toml** - Converting YAML to TOML format
- **yaml_parse_and_stringify** - Basic YAML parsing and output

## Resources

- [BitTorrent Protocol Specification](http://www.bittorrent.org/beps/bep_0003.html)
- [Bencode Specification](https://en.wikipedia.org/wiki/Bencode)
