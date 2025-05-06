import { Link } from 'react-router-dom';
import { useSelector } from 'react-redux';

const LandingPage = () => {
  const { isAuthenticated, user } = useSelector((state) => state.auth);

  return (
    <div className="bg-white">
      {/* Hero section */}
      <div className="relative isolate">
        <div className="mx-auto max-w-7xl px-6 py-16 lg:py-24">
          <div className="mx-auto max-w-3xl text-center">
            <h1 className="text-4xl font-bold tracking-tight text-gray-900 sm:text-6xl">
              Nu Scaler: Advanced Image Upscaling
            </h1>
            <p className="mt-6 text-lg leading-8 text-gray-600">
              Transform low-resolution images into crystal-clear visuals using our cutting-edge AI technology. 
              Experience unmatched quality and precision in every upscaled image.
            </p>
            <div className="mt-10 flex items-center justify-center gap-x-6">
              {isAuthenticated ? (
                <>
                  <Link
                    to="/download"
                    className="rounded-md bg-indigo-600 px-8 py-3 text-lg font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
                  >
                    Download Now
                  </Link>
                  {user?.is_admin && (
                    <Link
                      to="/admin"
                      className="rounded-md bg-red-600 px-8 py-3 text-lg font-semibold text-white shadow-sm hover:bg-red-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-600"
                    >
                      Admin Panel
                    </Link>
                  )}
                </>
              ) : (
                <>
                  <Link
                    to="/register"
                    className="rounded-md bg-indigo-600 px-6 py-3 text-lg font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
                  >
                    Get Started
                  </Link>
                  <Link
                    to="/login"
                    className="text-lg font-semibold leading-6 text-gray-900"
                  >
                    Sign in <span aria-hidden="true">→</span>
                  </Link>
                </>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Feature section */}
      <div className="bg-gray-50 py-16 sm:py-24">
        <div className="mx-auto max-w-7xl px-6 lg:px-8">
          <div className="mx-auto max-w-2xl lg:text-center">
            <h2 className="text-base font-semibold leading-7 text-indigo-600">Advanced Technology</h2>
            <p className="mt-2 text-3xl font-bold tracking-tight text-gray-900 sm:text-4xl">
              Superior Image Upscaling
            </p>
            <p className="mt-6 text-lg leading-8 text-gray-600">
              Nu Scaler combines multiple AI models to deliver unmatched image quality, preserving details that other upscalers miss.
            </p>
          </div>
          <div className="mx-auto mt-16 max-w-2xl sm:mt-20 lg:mt-24 lg:max-w-none">
            <dl className="grid max-w-xl grid-cols-1 gap-x-8 gap-y-16 lg:max-w-none lg:grid-cols-3">
              <div className="flex flex-col">
                <dt className="text-lg font-semibold leading-7 text-gray-900">
                  Ultra HD Resolution
                </dt>
                <dd className="mt-2 flex flex-auto flex-col text-base leading-7 text-gray-600">
                  <p className="flex-auto">
                    Scale images up to 4x their original size without losing quality or introducing artifacts.
                  </p>
                </dd>
              </div>
              <div className="flex flex-col">
                <dt className="text-lg font-semibold leading-7 text-gray-900">
                  Smart Detail Enhancement
                </dt>
                <dd className="mt-2 flex flex-auto flex-col text-base leading-7 text-gray-600">
                  <p className="flex-auto">
                    Our AI intelligently reconstructs fine details that were not visible in the original image.
                  </p>
                </dd>
              </div>
              <div className="flex flex-col">
                <dt className="text-lg font-semibold leading-7 text-gray-900">
                  Multi-platform Support
                </dt>
                <dd className="mt-2 flex flex-auto flex-col text-base leading-7 text-gray-600">
                  <p className="flex-auto">
                    Available for Windows, macOS, and Linux with both command-line and GUI options.
                  </p>
                </dd>
              </div>
            </dl>
          </div>
        </div>
      </div>
    </div>
  );
};

export default LandingPage; 