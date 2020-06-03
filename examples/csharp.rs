#![feature(proc_macro_hygiene)]

use genco::csharp::{local, using};
use genco::{quote, Csharp, Tokens};

use anyhow::Result;

fn main() -> Result<()> {
    let console = using("System", "Console");
    let file = using("System.IO", "File");
    let stream = using("System.IO", "Stream");
    let soap_formatter = using(
        "System.Runtime.Serialization.Formatters.Soap",
        "SoapFormatter",
    );
    let simple_object = local("TestSimpleObject");

    // Note: Comments have to be escaped as raw expressions, since they are
    // filtered out from procedural macros.
    let test: Tokens<Csharp> = quote! {
        public class Test {
            public static void Main()  {
               #("// Creates a new TestSimpleObject object.")
               #simple_object obj = new #simple_object();

               #console.WriteLine("Before serialization the object contains: ");
               obj.Print();

               #("// Opens a file and serializes the object into it in binary format.")
               #stream stream = #file.Open("data.xml", FileMode.Create);
               #soap_formatter formatter = new #soap_formatter();

               #("//BinaryFormatter formatter = new BinaryFormatter();")

               formatter.Serialize(stream, obj);
               stream.Close();

               #("// Empties obj.")
               obj = null;

               #("// Opens file \"data.xml\" and deserializes the object from it.")
               stream = #file.Open("data.xml", FileMode.Open);
               formatter = new #soap_formatter();

               #("//formatter = new BinaryFormatter();")

               obj = (#simple_object)formatter.Deserialize(stream);
               stream.Close();

               #console.WriteLine("");
               #console.WriteLine("After deserialization the object contains: ");
               obj.Print();
            }
         }
    };

    println!("{}", test.to_file_string()?);
    Ok(())
}
