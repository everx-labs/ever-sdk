pub const DEBOT_ABI: &'static str = r#"{
	"ABI version": 2,
	"header": ["pubkey", "time", "expire"],
	"functions": [
		{
			"name": "fetch",
			"inputs": [
			],
			"outputs": [
				{"components":[{"name":"id","type":"uint8"},{"name":"desc","type":"bytes"},{"components":[{"name":"desc","type":"bytes"},{"name":"name","type":"bytes"},{"name":"actionType","type":"uint8"},{"name":"attrs","type":"bytes"},{"name":"to","type":"uint8"},{"name":"misc","type":"cell"}],"name":"actions","type":"tuple[]"}],"name":"contexts","type":"tuple[]"}
			]
		},
		{
			"name": "start",
			"inputs": [
			],
			"outputs": [
			]
		},
		{
			"name": "quit",
			"inputs": [
			],
			"outputs": [
			]
		},
		{
			"name": "getVersion",
			"inputs": [
			],
			"outputs": [
				{"name":"name","type":"bytes"},
				{"name":"semver","type":"uint24"}
			]
		},
		{
			"name": "getErrorDescription",
			"inputs": [
				{"name":"error","type":"uint32"}
			],
			"outputs": [
				{"name":"desc","type":"bytes"}
			]
		},
		{
			"name": "getDebotOptions",
			"inputs": [
			],
			"outputs": [
				{"name":"options","type":"uint8"},
				{"name":"debotAbi","type":"bytes"},
				{"name":"targetAbi","type":"bytes"},
				{"name":"targetAddr","type":"address"}
			]
		},
		{
			"name": "constructor",
			"inputs": [
			],
			"outputs": [
			]
		}
	],
	"data": [
	],
	"events": [
	]
}"#;