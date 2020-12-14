pub const CONSOLE_ABI: &'static str = r#"{
  "ABI version": 2,
  "header": [
    "pubkey",
    "time",
    "expire"
  ],
  "functions": [
  {
    "name": "print",
    "inputs": [
    { "name":"_answer_id", "type":"uint32" },
    { "name":"message", "type":"bytes" }
    ],
    "outputs": [
    { "name":"value0", "type":"bool" }
    ]
  },
  {
    "name": "printf",
    "inputs": [
    { "name":"_answer_id", "type":"uint32" },
    { "name":"message", "type":"bytes" },
    { "name":"params", "type":"cell" }
    ],
    "outputs": [
    { "name":"value0", "type":"bool" }
    ]
  },
  {
    "name": "inputStr",
    "inputs": [
    { "name":"_answer_id", "type":"uint32" },
    { "name":"message", "type":"bytes" }
    ],
    "outputs": [
    { "name":"value0", "type":"bytes" }
    ]
  },
  {
    "name": "inputAddress",
    "inputs": [
    { "name":"_answer_id", "type":"uint32" },
    { "name":"message", "type":"bytes" },
    { "name":"key_hint", "type":"bytes" }
    ],
    "outputs": [
    { "name":"value0", "type":"address" }
    ]
  },
  {
    "name": "inputUint256",
    "inputs": [
    { "name":"_answer_id", "type":"uint32" },
    { "name":"message", "type":"bytes" }
    ],
    "outputs": [
    { "name":"value0", "type":"uint256" }
    ]
  },
  {
    "name": "inputPubkey",
    "inputs": [
    { "name":"_answer_id", "type":"uint32" },
    { "name":"message", "type":"bytes" }
    ],
    "outputs": [
    { "name":"value0", "type":"uint256" }
    ]
  },
  {
    "name": "inputTONs",
    "inputs": [
    { "name":"_answer_id", "type":"uint32" },
    { "name":"message", "type":"bytes" }
    ],
    "outputs": [
    { "name":"value0", "type":"uint256" }
    ]
  },
  {
    "name": "inputYesOrNo",
    "inputs": [
    { "name":"_answer_id", "type":"uint32" },
    { "name":"message", "type":"bytes" }
    ],
    "outputs": [
    { "name":"value0", "type":"bool" }
    ]
  },
  {
    "name": "inputDateTime",
    "inputs": [
    { "name":"_answer_id", "type":"uint32" },
    { "name":"message", "type":"bytes" }
    ],
    "outputs": [
    { "name":"value0", "type":"uint32" }
    ]
  },
  {
    "name": "inputDeployMessage",
    "inputs": [
    { "name":"_answer_id", "type":"uint32" },
    { "name":"message", "type":"bytes" }
    ],
    "outputs": [
    { "name":"value0", "type":"cell" }
    ]
  },
  {
    "name": "inputCell",
    "inputs": [
    { "name":"_answer_id", "type":"uint32" },
    { "name":"message", "type":"bytes" }
    ],
    "outputs": [
    { "name":"value0", "type":"cell" }
    ]
  },
  {
    "name": "iAmHome",
    "inputs": [
    { "name":"_answer_id", "type":"uint32" }
    ],
    "outputs": [
    { "name":"value0", "type":"bool" }
    ]
  }
  ],
  "events": [
  ]
}"#;
