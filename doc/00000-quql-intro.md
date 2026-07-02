# QuQL

A query language that will be the only one we'll support cuz I just don't
care to do any others.

It's very similar to SQL because it works.

Just tables:

```
create table that_table fields {
    id: uuid id default { uuid::rand() },
    a: i64,
    b: str,
};

insert that_table fields {
    a: 2,
    b: "hello",
};

select * from that_table where a = 2;

delete from that_table where a = 2;

# We can just reuse `delete` since `from` is required to delete rows anyway.
delete table that_table;
```

Message queue:

```
create table that_queue ephemeral ttl 10s fields {
    id: uuid id default { uuid::rand() },
    greeting: str,
};

select * from that_queue live omit id;

insert that_queue fields {
    greeting: "hello",
};
```

Graphs cuz why not:

```
create table edge fields {
    id: uuid id default { uuid::rand() },
};
```

Eval:

```
# By default nothing is exported to eval
eval "select * from that_table";

# So you can export stuff explicitly
eval "select * from that_table where a = $a" with $a = 2;

# You can also compile and run qubcl code. This is how it works internally anyway
eval quql::compile("select * from that_table");
```

Namespaces, databases:

```
use ns/db;
```

Mutexes:

```
create table
```

## QuBQL

QuQL but binary.

QuBQL allows sending ASTs to the server instead of hard-to-escape strings. Also useful for query builders.
