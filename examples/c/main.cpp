#include <iostream>
#include "tonclient.h"


int main() {
    auto context = CoreContext(R"({ "servers": "http://localhost:8080" })");
    auto version = context.request("client.version", "");
    cout << version << endl;
    return 0;
}

