use gameboy_emulator::screen::{Screen, PixelsScreen};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

fn main() {
    let event_loop = EventLoop::new();
    let mut screen = PixelsScreen::new(256, 256, 512, 512, &event_loop);

    let mut index = 0;
    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::WindowEvent { 
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("Closing");
                control_flow.set_exit();
            },
            Event::MainEventsCleared => {
                dbg!(index);
                for r in 0u8..=255 {
                    for g in 0u8..=255 {
                        let row = r.wrapping_add((3 * index) as u8);
                        let col = g.wrapping_sub(index as u8);
                        screen.set_pixel(row.into(), col.into(), &[r, g, 0, 255]).unwrap();
                    }
                }
                index = (index + 1) % 256;

                screen.redraw();
            },
            Event::RedrawRequested(_) => {

            },
            _ => ()
        }
    });
}
