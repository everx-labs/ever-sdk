pub const DEBOT_ABI: &'static str = r#"{
	"ABI version": 2,
    "version": "2.2",
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
			"name": "getDebotInfo",
            "id": "0xDEB",
			"inputs": [
			],
			"outputs": [
				{"name":"name","type":"string"},
				{"name":"version","type":"string"},
				{"name":"publisher","type":"string"},
				{"name":"caption","type":"string"},
				{"name":"author","type":"string"},
				{"name":"support","type":"address"},
				{"name":"hello","type":"string"},
				{"name":"language","type":"string"},
				{"name":"dabi","type":"string"},
				{"name":"icon","type":"string"}

			]
		},
        {
            "name": "getRequiredInterfaces",
            "inputs": [
            ],
            "outputs": [
                    {"name":"interfaces","type":"uint256[]"}
            ]
        }
	],
	"data": [
	],
	"events": [
	]
}"#;