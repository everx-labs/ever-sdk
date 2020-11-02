ton-client-jni
========================
JNI wrapper library for `tonclient.so`, `tonclient.dll`, `tonclient.dynlib`.

### Building

Environmental Variable `JAVA_HOME` must be set for header `jni.h` to resolve.<br/>
The Oracle JDK has a different file-system layout (it won't resolve the header). 

Linux/Darwin:

    export JAVA_HOME=$HOME/.jdks/corretto-1.8.0_265
    
Windows:

    %USERPROFILE%\.jdks\corretto-1.8.0_265

Build with: `./gradlew assemble` or `gradlew.bat assemble`.

Run task `:updateJniBindings` after having built once.

### Testing

Test with: `./gradlew test` or `gradlew.bat test`.

### Know Issues

When it doesn't link on Windows, add `C:\Windows\System32\ton_client.dll`. Also `del %USERPROFILE%\AppData\Local\Temp\tonclient*` may help.