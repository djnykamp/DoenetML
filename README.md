# Doenet in Rust
 
## Quickstart
In the project folder:

`$ npm install`

`$ npm run compile-wasm`

`$ npm run dev`

Paste demo code into `src/test/testCode.doenet`

Navigate to `localhost:3000/src/test/index.html`

## Demos

<details>
<summary>Point and Collect</summary>

``` xml
<graph name="graph">
	<point name="p1" xs="2 3"/>
	<point name="p2" xs="$p1.y $p1.x"/>
</graph>
<text>
	<collect source="graph" componentType="point"/>
</text>
```
</details>

<details>
<summary>Text Input</summary>

``` xml
<textInput name="t1" prefill="Cake"/>
<text>$t1.value is good.</text>
```
</details>

<details>
<summary>Sequence and Number Input</summary>

``` xml
<numberInput name="n1" prefill="4"/>
<numberInput name="n2" prefill="14"/>
<p>
	Count from $n1.value to $n2.value:
	<sequence name="seq" from="$n1.value" to="$n2.value"/>.

	And the fifth number is $seq[5].value.
</p>
```
</details>

<details>
<summary>Point Parralellogram</summary>

``` xml
<graph>
	<point name="p1" xs="0 4"/>
	<point name="p2" xs="3 0"/>
	<point name="p3" xs="$p1.x+$p2.x $p1.y+$p2.y"/>
</graph>
```
</details>

<details>
<summary>Boolean Input</summary>

``` xml
<booleanInput name="bool"/>

I think<text hide="$bool.value"> therefore I am</text>.

<booleanInput name="bool2"/>
<text hide="$bool2.value">Yin</text>
<text hide="$!bool2.value">Yang</text>
```
</details>

<details>
<summary>Value vs Immediate Value</summary>

``` xml
<graph name="graph">
	<point name="p1" xs="$n1.value $n2.value"/>
	<point name="p2" xs="$n1.immediateValue+0.5 $n2.immediateValue"/>
</graph>

<numberInput name="n1" prefill="0"/>
<numberInput name="n2" prefill="0"/>

One point uses immediate value plus an offset
```
</details>

---untested past this point---

<details>
<summary>Collect Component Index</summary>

``` xml
<p name="p1">
This paragraph has numbers and sequences:

<number>23</number>
and
<sequence from="1" to="$n"/>
<number>42</number>
and more
<number>2</number>
<sequence from="$n" to="2*$n"/>
last one
<number>30</number>
</p>

Collect the numbers in that paragraph: <collect name="c1" source="p1" componentType="number"/>

The fifth number is $c1[5].value.

Now try changing the number input
<numberInput prefill="6"/><number name="n" copySource="/_numberInput" copyProp="value/>
```
</details>

## Technical Documentation
JavaScript parses the DoenetML and call Rust functions, passing in strings. On core creation, Rust returns a pointer to its main struct, existing in WASM linear memory. Javascript uses this to access the other core functions. Rust returns rendering data as strings.

The Doenet Rust code is in the doenet-core crate, doenet-core/src/lib.rs being the main file. The crate can be built as a library independent of javascript, but without a parser, one would need pre-parsed DoenetML objects as its input.

