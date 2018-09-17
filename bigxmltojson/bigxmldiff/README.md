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

