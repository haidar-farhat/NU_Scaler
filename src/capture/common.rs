use image::imageops; 

Ok(imageops::resize(input, width, height, imageops::FilterType::Lanczos3)) 