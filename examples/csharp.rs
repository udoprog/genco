use csharp::comment;
use genco::fmt;
use genco::prelude::*;

fn main() -> anyhow::Result<()> {
    let console = &csharp::import("System", "Console");
    let file = &csharp::import("System.IO", "File");
    let stream = &csharp::import("System.IO", "Stream");
    let soap_formatter = &csharp::import(
        "System.Runtime.Serialization.Formatters.Soap",
        "SoapFormatter",
    );
    let simple_object = "TestSimpleObject";

    // Note: Comments have to be escaped as raw expressions, since they are
    // filtered out from procedural macros.
    let tokens = quote! {
        public class Test {
            public static void Main()  {
                $(comment(&["Creates a new TestSimpleObject object."]))
                $simple_object obj = new $simple_object();

                $console.WriteLine("Before serialization the object contains: ");
                obj.Print();

                $(comment(&["Opens a file and serializes the object into it in binary format."]))
                $stream stream = $file.Open("data.xml", FileMode.Create);
                $soap_formatter formatter = new $soap_formatter();

                $(comment(&["BinaryFormatter formatter = new BinaryFormatter();"]))

                formatter.Serialize(stream, obj);
                stream.Close();

                $(comment(&["Empties obj."]))
                obj = null;

                $(comment(&["Opens file \"data.xml\" and deserializes the object from it."]))
                stream = $file.Open("data.xml", FileMode.Open);
                formatter = new $soap_formatter();

                $(comment(&["formatter = new BinaryFormatter();"]))

                obj = ($simple_object)formatter.Deserialize(stream);
                stream.Close();

                $console.WriteLine("");
                $console.WriteLine("After deserialization the object contains: ");
                obj.Print();
            }
        }
    };

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt = fmt::Config::from_lang::<Csharp>().with_indentation(fmt::Indentation::Space(4));
    let config = csharp::Config::default();

    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
    Ok(())
}
