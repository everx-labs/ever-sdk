use std::io;
use tvm::types::Exception;

error_chain! {

    types {
        SdkError, SdkErrorKind, SdkResultExt, SdkResult;
    }

    foreign_links {
        Io(io::Error);
        Tvm(Exception);
        DB(reql::errors::Error);
    }

    errors {
        NotFound {
            description("Requested item not found")
        }
        DataBaseProblem {
            description("Database problem")
        }        
        InvalidOperation {
            description("Invalid operation")
        }
    }

}