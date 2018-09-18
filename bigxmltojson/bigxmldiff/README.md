# Big XML diff
The purpose of this test is to compare two big XMLs that do not fit in memory.

The XML structure must be understood so that chunks and data keys can be
recognized for further comparison with other files.

## Example data
```xml
<ITEMS>
<ITEM>
<ITEM_KEY_1>1</ITEM_KEY_1>
<ITEM_KEY_2>B</ITEM_KEY_2>
<DATA>
<ITEM_DATA_1>SomeData<ITEM_DATA_1>
<ITEM_DATA_2>SomeOtherData<ITEM_DATA_2>
</DATA>
</ITEM>
...
<ITEM>
<ITEM_KEY_1>1</ITEM_KEY_1>
<ITEM_KEY_2>B</ITEM_KEY_2>
</ITEM>
</ITEMS>
```

## Chunking
The file is read once, in a configurable `chunk_size` bytes.
The XML goes through minimal parsing to find the chunk start and end offsets.

In the example data, the Chunk Separator is `<ITEM>`, and it would be used for
recognize when data is modified, added or deleted.

## Concurrent Key Identification and Checksum Calculation
A ZMQ Router binds to a `bind_address` to distribute tasks.
Worker threads (ZMQ Dealer) configured through `concurrency` are started and
connect to the `bind_address` for work. As soon as they finish a new task is
started for them.
The worker processes each chunk, find its key(s), then it sorts the XML lines
(In case the data remains the same but the order differs. One caveat is that
the records must be line separated)
Once the XML chunk lines are sorted, a sha1sum operation is performed on it.
The found keys, shasum and offset are returned to the broker.

## Indexing
The data returned to the broker thread are added to a BTreeMap. Containing:
- Offset
- Key
- Checksum

## Method of comparison
An .idx file is created for each XML and the added/deleted/modified entries
are recognized based on the keys.
To leverage the reading of the file sequentially, fseek is isued on the stored
offset, a sorted versio of the added/deleted/modified calculation should be
cheap to perform.

## Delta XML chunk offset reader
Once a chunk has been identified as changed, the chunk is found from the XML
based on the offset. The chunk is sent to as a task to the ZMQ Router.

## JSON serializing
Based on `concurrency`, several threads are spawned. Each reads a chunk and
transforms it into a JSON reading the XML little by little.

## Future thoughts
A state machine
Several State Machines as the OCW601 exercises can be implemented out of this:
- A big file reader in chunks.
  - Input: A "big" file, an chunk size.
  - State: Offset
  - Output: A chunk string. Is an offset end needed?
- An XML chunk key identifier.
  - Input: A chunk string
  - State: None
  - Output: The ID of the XML chunk.
- An XML chunk checksum calculator.
  - Input: A chunk string
  - State: None
  - Output: The checksum of the XML chunk.
- The ZMQ Local Thread Router task distributor, eventually replaced by Tokio.
  - Input: A "task".
  - State:
    - The threads
    - The vector of tasks, maybe ZMTP.
    - The UUI of each task for the client requests.
  - Output: The result of the tasks.
- A ZMQ Router State Machine that binds somewhere
  - Input: A task
  - State: Depends on each module.
  - Output: Returns something to ZMQ Router.
