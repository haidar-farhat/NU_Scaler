import { useState } from 'react';
import testimonials from '../data/testimonials';

const TestimonialsCarousel = () => {
  const [index, setIndex] = useState(0);
  const next = () => setIndex((i) => (i + 1) % testimonials.length);
  const prev = () => setIndex((i) => (i - 1 + testimonials.length) % testimonials.length);

  return (
    <div className="max-w-xl mx-auto mt-10">
      <div className="bg-white p-6 rounded shadow text-center">
        <p className="text-lg italic mb-2">"{testimonials[index].quote}"</p>
        <div className="text-sm text-gray-500">- {testimonials[index].author}</div>
        <div className="flex justify-center gap-2 mt-4">
          <button onClick={prev} className="px-2 py-1 bg-gray-200 rounded hover:bg-gray-300">&lt;</button>
          <button onClick={next} className="px-2 py-1 bg-gray-200 rounded hover:bg-gray-300">&gt;</button>
        </div>
      </div>
    </div>
  );
};
export default TestimonialsCarousel; 