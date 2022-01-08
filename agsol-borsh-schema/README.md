## Description
A parsing library that generates [TypeScript classes and serialization
schemas](https://github.com/near/borsh-js) from Rust data structures.

## Usage
Prepend a Rust `struct` with the `BorshSchema` derivable trait like this
```rust
#[derive(BorshSchema)]
struct SomeStruct {
	foo: u32,
	bar: Option<u64>,
	baz: Vec<String>,
	quux: BTreeMap<[u8; 32], Pubkey>,
}
```
and the parser will generate the following TypeScript output:

```ts
export class SomeStruct extends Struct {
	foo: number,
	bar: BN | null,
	baz: string[],
	quux: Map<[32], PublicKey>,
}

export const SCHEMA = new Map<any, any>([
	[
		SomeStruct,
		{
			kind: 'struct', fields [
				['foo', 'u32'],
				['bar', { kind: 'option', type: 'u64' }],
				['baz', ['string']],
				['quux', { kind: 'map', key: [32], value: 'publicKey' }],
			],
		},
	],
])
```
The library also supports `enum` types, just add the `BorshSchema` derive attribute.

For example 
```rust
#[derive(BorshSchema)]
struct FooStruct {
	foo: Option<String>,
}
#[derive(BorshSchema)]
enum SomeEnum {
	UnitVariant,
	UnnamedFields(u64, [String; 2]),
	NamedFields {
		foo_struct: FooStruct,
		bar: Vec<u8>,
	},
}
```
will result in
```ts
export class FooStruct extends Struct {
	foo: string | null,
}

export class SomeEnum extends Enum {
	someEnumUnitVariant: someEnumUnitVariant,
	someEnumUnnamedFields: someEnumUnnamedFields, 
	someEnumNamedFields: someEnumNamedFields, 
}

export class SomeEnumUnitVariant extends Struct {}
export class SomeEnumUnnamedFields extends Struct {
	unnamed_1: BN,
	unnamed_2: string[],
}

export class SomeEnumNamedFields extends Struct {
	fooStruct: FooStruct,
	bar: number[],
}

export const SCHEMA = new Map<any, any>([
	[
		FooStruct,
		{
			kind: 'struct', fields [
				[foo: { kind: 'option', type: 'u64' }],
			],
		},
	],
	[
		SomeEnum,
		{
			kind: 'enum', field: 'enum', values: [
				['someEnumUnitVariant', SomeEnumUnitVariant],
				['someEnumUnnamedFields', SomeEnumUnnamedFields],
				['someEnumNamedFields', SomeEnumNnamedFields],
			],
		},
	],
	[
		SomeEnumUnitVariant,
		{
			kind: `struct`, fields [],
		},
	],
	[
		SomeEnumUnnamedFields,
		{
			kind: `struct`, fields [
				['unnamed_1', u64],
				['unnamed_2', ['string', 2]],
			],
		},
	],
	[
		SomeEnumNamedFields,
		{
			kind: `struct`, fields [
				['fooStruct', FooStruct],
				['bar', ['u8']],
			],
		},
	],
])
```
