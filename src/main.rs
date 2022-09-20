extern crate lyon;
use lyon::math::{point, Point};
use lyon::path::builder::*;
use lyon::path::Path;
use lyon::tessellation::*;

// Let's use our own custom vertex type instead of the default one.
#[derive(Copy, Clone, Debug)]
struct MyVertex {
    position: [f32; 2],
}

fn main() {
    // // Build a Path.
    // let mut builder = Path::builder();
    // builder.begin(point(0.0, 0.0));
    // // builder.line_to(point(1.0, 0.0));
    // // builder.quadratic_bezier_to(point(2.0, 0.0), point(2.0, 1.0));
    // builder.cubic_bezier_to(point(1.0, 1.0), point(0.0, 1.0), point(10.0, 0.0));
    // builder.close();
    // let path = builder.build();

    // // Will contain the result of the tessellation.
    // let mut geometry: VertexBuffers<MyVertex, u16> = VertexBuffers::new();

    // let mut tessellator = FillTessellator::new();

    // {
    //     // Compute the tessellation.
    //     tessellator
    //         .tessellate_path(
    //             &path,
    //             &FillOptions::default(),
    //             &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| MyVertex {
    //                 position: vertex.position().to_array(),
    //             }),
    //         )
    //         .unwrap();
    // }

    // eprintln!("{:#?}", geometry);

    // // The tessellated geometry is ready to be uploaded to the GPU.
    // println!(
    //     " -- {} vertices {} indices",
    //     geometry.vertices.len(),
    //     geometry.indices.len()
    // );

    let mut builder = Path::builder();

    // All sub-paths *must* have be contained in a being/end pair.
    builder.begin(point(0.0, 0.0));
    builder.line_to(point(1.0, 0.0));
    builder.quadratic_bezier_to(point(2.0, 0.0), point(2.0, 1.0));
    builder.end(false);

    builder.begin(point(10.0, 0.0));
    builder.cubic_bezier_to(point(12.0, 2.0), point(11.0, 2.0), point(5.0, 0.0));
    builder.close(); // close() is equivalent to end(true).

    let path = builder.build();

    // Will contain the result of the tessellation.
    let mut geometry: VertexBuffers<MyVertex, u16> = VertexBuffers::new();

    let mut tessellator = FillTessellator::new();

    {
        // Compute the tessellation.
        tessellator
            .tessellate_path(
                &path,
                &FillOptions::default(),
                &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| MyVertex {
                    position: vertex.position().to_array(),
                }),
            )
            .unwrap();
    }

    eprintln!("{:#?}", geometry);

    // The tessellated geometry is ready to be uploaded to the GPU.
    println!(
        " -- {} vertices {} indices",
        geometry.vertices.len(),
        geometry.indices.len()
    );
}
