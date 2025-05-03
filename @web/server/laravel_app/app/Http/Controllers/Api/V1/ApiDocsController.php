<?php

namespace App\Http\Controllers\Api\V1;

use App\Http\Controllers\Controller;

/**
 * @OA\Info(
 *     title="Nu Scaler API",
 *     version="1.0.0",
 *     description="Nu Scaler API Documentation",
 *     @OA\Contact(
 *         email="support@nuscaler.com",
 *         name="Nu Scaler Support"
 *     ),
 *     @OA\License(
 *         name="MIT",
 *         url="https://opensource.org/licenses/MIT"
 *     )
 * )
 *
 * @OA\Server(
 *     url="/api",
 *     description="Nu Scaler API Server"
 * )
 *
 * @OA\SecurityScheme(
 *     securityScheme="bearerAuth",
 *     type="http",
 *     scheme="bearer",
 *     bearerFormat="JWT"
 * )
 *
 * @OA\Tag(
 *     name="Authentication",
 *     description="API endpoints for user authentication"
 * )
 *
 * @OA\Tag(
 *     name="Feedback",
 *     description="API endpoints for submitting feedback"
 * )
 *
 * @OA\Tag(
 *     name="Admin",
 *     description="API endpoints for admin operations"
 * )
 *
 * @OA\Schema(
 *     schema="Review",
 *     required={"rating", "comment"},
 *     @OA\Property(property="id", type="integer", format="int64", example=1),
 *     @OA\Property(property="rating", type="integer", format="int32", example=5),
 *     @OA\Property(property="comment", type="string", example="Great product!"),
 *     @OA\Property(property="name", type="string", nullable=true, example="John Doe"),
 *     @OA\Property(property="email", type="string", format="email", nullable=true, example="john@example.com"),
 *     @OA\Property(property="user_uuid", type="string", format="uuid", nullable=true, example="550e8400-e29b-41d4-a716-446655440000"),
 *     @OA\Property(property="created_at", type="string", format="date-time"),
 *     @OA\Property(property="updated_at", type="string", format="date-time")
 * )
 *
 * @OA\Schema(
 *     schema="BugReport",
 *     required={"description", "category", "severity", "system_info"},
 *     @OA\Property(property="id", type="integer", format="int64", example=1),
 *     @OA\Property(property="description", type="string", example="Application crashes when processing large images"),
 *     @OA\Property(property="category", type="string", enum={"ui", "performance", "feature", "crash", "other"}, example="crash"),
 *     @OA\Property(property="severity", type="string", enum={"low", "medium", "high", "critical"}, example="high"),
 *     @OA\Property(property="steps_to_reproduce", type="string", nullable=true, example="1. Open a large image\n2. Apply filter\n3. Click process"),
 *     @OA\Property(
 *         property="system_info",
 *         type="object",
 *         @OA\Property(property="os", type="string", example="Windows 11"),
 *         @OA\Property(property="browser", type="string", nullable=true, example="Chrome 92.0"),
 *         @OA\Property(property="device", type="string", nullable=true, example="Desktop"),
 *         @OA\Property(property="app_version", type="string", example="1.2.0")
 *     ),
 *     @OA\Property(property="user_uuid", type="string", format="uuid", nullable=true, example="550e8400-e29b-41d4-a716-446655440000"),
 *     @OA\Property(property="created_at", type="string", format="date-time"),
 *     @OA\Property(property="updated_at", type="string", format="date-time")
 * )
 *
 * @OA\Schema(
 *     schema="HardwareSurvey",
 *     required={"cpu_model", "gpu_model", "ram_size", "os", "resolution"},
 *     @OA\Property(property="id", type="integer", format="int64", example=1),
 *     @OA\Property(property="cpu_model", type="string", example="Intel Core i7-12700K"),
 *     @OA\Property(property="gpu_model", type="string", example="NVIDIA RTX 3080"),
 *     @OA\Property(property="ram_size", type="integer", format="int32", example=32),
 *     @OA\Property(property="os", type="string", example="Windows 11"),
 *     @OA\Property(property="resolution", type="string", example="3840x2160"),
 *     @OA\Property(property="monitor_refresh_rate", type="integer", format="int32", nullable=true, example=144),
 *     @OA\Property(property="additional_info", type="string", nullable=true, example="Using dual monitor setup"),
 *     @OA\Property(property="user_uuid", type="string", format="uuid", nullable=true, example="550e8400-e29b-41d4-a716-446655440000"),
 *     @OA\Property(property="created_at", type="string", format="date-time"),
 *     @OA\Property(property="updated_at", type="string", format="date-time")
 * )
 */
class ApiDocsController extends Controller
{
    /**
     * @OA\Post(
     *     path="/v1/feedback/reviews",
     *     operationId="storeReview",
     *     tags={"Feedback"},
     *     summary="Submit a review",
     *     description="Submit a new review for Nu Scaler",
     *     @OA\RequestBody(
     *         required=true,
     *         @OA\JsonContent(
     *             required={"rating", "comment"},
     *             @OA\Property(property="rating", type="integer", format="int32", example=5, description="Rating from 1 to 5"),
     *             @OA\Property(property="comment", type="string", example="Great product!", description="Review comment"),
     *             @OA\Property(property="name", type="string", nullable=true, example="John Doe", description="Reviewer's name (optional)"),
     *             @OA\Property(property="email", type="string", format="email", nullable=true, example="john@example.com", description="Reviewer's email (optional)")
     *         )
     *     ),
     *     @OA\Response(
     *         response=201,
     *         description="Review created successfully",
     *         @OA\JsonContent(
     *             @OA\Property(property="message", type="string", example="Review submitted successfully"),
     *             @OA\Property(property="data", ref="#/components/schemas/Review")
     *         )
     *     ),
     *     @OA\Response(
     *         response=422,
     *         description="Validation error",
     *         @OA\JsonContent(
     *             @OA\Property(property="message", type="string", example="The given data was invalid."),
     *             @OA\Property(
     *                 property="errors",
     *                 type="object",
     *                 @OA\Property(
     *                     property="rating",
     *                     type="array",
     *                     @OA\Items(type="string", example="The rating field is required.")
     *                 )
     *             )
     *         )
     *     )
     * )
     */

    /**
     * @OA\Post(
     *     path="/v1/feedback/bug-reports",
     *     operationId="storeBugReport",
     *     tags={"Feedback"},
     *     summary="Submit a bug report",
     *     description="Submit a new bug report for Nu Scaler",
     *     @OA\RequestBody(
     *         required=true,
     *         @OA\JsonContent(
     *             required={"description", "category", "severity", "system_info"},
     *             @OA\Property(property="description", type="string", example="Application crashes when processing large images"),
     *             @OA\Property(property="category", type="string", enum={"ui", "performance", "feature", "crash", "other"}, example="crash"),
     *             @OA\Property(property="severity", type="string", enum={"low", "medium", "high", "critical"}, example="high"),
     *             @OA\Property(property="steps_to_reproduce", type="string", nullable=true, example="1. Open a large image\n2. Apply filter\n3. Click process"),
     *             @OA\Property(
     *                 property="system_info",
     *                 type="object",
     *                 required={"os", "app_version"},
     *                 @OA\Property(property="os", type="string", example="Windows 11"),
     *                 @OA\Property(property="browser", type="string", nullable=true, example="Chrome 92.0"),
     *                 @OA\Property(property="device", type="string", nullable=true, example="Desktop"),
     *                 @OA\Property(property="app_version", type="string", example="1.2.0")
     *             )
     *         )
     *     ),
     *     @OA\Response(
     *         response=201,
     *         description="Bug report created successfully",
     *         @OA\JsonContent(
     *             @OA\Property(property="message", type="string", example="Bug report submitted successfully"),
     *             @OA\Property(property="data", ref="#/components/schemas/BugReport")
     *         )
     *     ),
     *     @OA\Response(
     *         response=422,
     *         description="Validation error",
     *         @OA\JsonContent(
     *             @OA\Property(property="message", type="string", example="The given data was invalid."),
     *             @OA\Property(
     *                 property="errors",
     *                 type="object",
     *                 @OA\Property(
     *                     property="description",
     *                     type="array",
     *                     @OA\Items(type="string", example="The description field is required.")
     *                 )
     *             )
     *         )
     *     )
     * )
     */

    /**
     * @OA\Post(
     *     path="/v1/feedback/hardware-surveys",
     *     operationId="storeHardwareSurvey",
     *     tags={"Feedback"},
     *     summary="Submit a hardware survey",
     *     description="Submit a new hardware survey for Nu Scaler",
     *     @OA\RequestBody(
     *         required=true,
     *         @OA\JsonContent(
     *             required={"cpu_model", "gpu_model", "ram_size", "os", "resolution"},
     *             @OA\Property(property="cpu_model", type="string", example="Intel Core i7-12700K"),
     *             @OA\Property(property="gpu_model", type="string", example="NVIDIA RTX 3080"),
     *             @OA\Property(property="ram_size", type="integer", format="int32", example=32),
     *             @OA\Property(property="os", type="string", example="Windows 11"),
     *             @OA\Property(property="resolution", type="string", example="3840x2160"),
     *             @OA\Property(property="monitor_refresh_rate", type="integer", format="int32", nullable=true, example=144),
     *             @OA\Property(property="additional_info", type="string", nullable=true, example="Using dual monitor setup")
     *         )
     *     ),
     *     @OA\Response(
     *         response=201,
     *         description="Hardware survey created successfully",
     *         @OA\JsonContent(
     *             @OA\Property(property="message", type="string", example="Hardware survey submitted successfully"),
     *             @OA\Property(property="data", ref="#/components/schemas/HardwareSurvey")
     *         )
     *     ),
     *     @OA\Response(
     *         response=422,
     *         description="Validation error",
     *         @OA\JsonContent(
     *             @OA\Property(property="message", type="string", example="The given data was invalid."),
     *             @OA\Property(
     *                 property="errors",
     *                 type="object",
     *                 @OA\Property(
     *                     property="gpu_model",
     *                     type="array",
     *                     @OA\Items(type="string", example="The gpu_model field is required.")
     *                 )
     *             )
     *         )
     *     )
     * )
     */

    /**
     * @OA\Get(
     *     path="/admin/metrics/dashboard",
     *     operationId="getDashboardMetrics",
     *     tags={"Admin"},
     *     summary="Get dashboard metrics",
     *     description="Get summary metrics for the admin dashboard",
     *     security={{"bearerAuth": {}}},
     *     @OA\Response(
     *         response=200,
     *         description="Success",
     *         @OA\JsonContent(
     *             @OA\Property(
     *                 property="data",
     *                 type="object",
     *                 @OA\Property(
     *                     property="users",
     *                     type="object",
     *                     @OA\Property(property="total", type="integer", example=150),
     *                     @OA\Property(property="new_today", type="integer", example=12)
     *                 ),
     *                 @OA\Property(
     *                     property="reviews",
     *                     type="object",
     *                     @OA\Property(property="total", type="integer", example=75),
     *                     @OA\Property(property="average_rating", type="number", format="float", example=4.2),
     *                     @OA\Property(property="new_today", type="integer", example=5)
     *                 ),
     *                 @OA\Property(
     *                     property="bug_reports",
     *                     type="object",
     *                     @OA\Property(property="total", type="integer", example=30),
     *                     @OA\Property(property="new_today", type="integer", example=3),
     *                     @OA\Property(
     *                         property="by_severity",
     *                         type="object",
     *                         @OA\Property(property="low", type="integer", example=10),
     *                         @OA\Property(property="medium", type="integer", example=12),
     *                         @OA\Property(property="high", type="integer", example=5),
     *                         @OA\Property(property="critical", type="integer", example=3)
     *                     )
     *                 ),
     *                 @OA\Property(
     *                     property="hardware_surveys",
     *                     type="object",
     *                     @OA\Property(property="total", type="integer", example=50),
     *                     @OA\Property(property="new_today", type="integer", example=2)
     *                 )
     *             )
     *         )
     *     ),
     *     @OA\Response(
     *         response=401,
     *         description="Unauthenticated",
     *         @OA\JsonContent(
     *             @OA\Property(property="message", type="string", example="Unauthenticated")
     *         )
     *     ),
     *     @OA\Response(
     *         response=403,
     *         description="Forbidden",
     *         @OA\JsonContent(
     *             @OA\Property(property="message", type="string", example="Unauthorized. Admin access required.")
     *         )
     *     )
     * )
     */

    /**
     * @OA\Get(
     *     path="/admin/reviews",
     *     operationId="getReviews",
     *     tags={"Admin"},
     *     summary="Get all reviews",
     *     description="Get a paginated list of all reviews",
     *     security={{"bearerAuth": {}}},
     *     @OA\Parameter(
     *         name="page",
     *         in="query",
     *         description="Page number",
     *         required=false,
     *         @OA\Schema(type="integer", default=1)
     *     ),
     *     @OA\Parameter(
     *         name="per_page",
     *         in="query",
     *         description="Number of items per page",
     *         required=false,
     *         @OA\Schema(type="integer", default=15)
     *     ),
     *     @OA\Parameter(
     *         name="rating",
     *         in="query",
     *         description="Filter by rating",
     *         required=false,
     *         @OA\Schema(type="integer", enum={1, 2, 3, 4, 5})
     *     ),
     *     @OA\Parameter(
     *         name="search",
     *         in="query",
     *         description="Search in comments",
     *         required=false,
     *         @OA\Schema(type="string")
     *     ),
     *     @OA\Response(
     *         response=200,
     *         description="Success",
     *         @OA\JsonContent(
     *             @OA\Property(property="current_page", type="integer", example=1),
     *             @OA\Property(property="data", type="array", @OA\Items(ref="#/components/schemas/Review")),
     *             @OA\Property(property="first_page_url", type="string", example="http://example.com/api/admin/reviews?page=1"),
     *             @OA\Property(property="from", type="integer", example=1),
     *             @OA\Property(property="last_page", type="integer", example=5),
     *             @OA\Property(property="last_page_url", type="string", example="http://example.com/api/admin/reviews?page=5"),
     *             @OA\Property(property="links", type="array",
     *                 @OA\Items(
     *                     type="object",
     *                     @OA\Property(property="url", type="string", nullable=true, example="http://example.com/api/admin/reviews?page=1"),
     *                     @OA\Property(property="label", type="string", example="1"),
     *                     @OA\Property(property="active", type="boolean", example=true)
     *                 )
     *             ),
     *             @OA\Property(property="next_page_url", type="string", nullable=true, example="http://example.com/api/admin/reviews?page=2"),
     *             @OA\Property(property="path", type="string", example="http://example.com/api/admin/reviews"),
     *             @OA\Property(property="per_page", type="integer", example=15),
     *             @OA\Property(property="prev_page_url", type="string", nullable=true, example=null),
     *             @OA\Property(property="to", type="integer", example=15),
     *             @OA\Property(property="total", type="integer", example=75)
     *         )
     *     ),
     *     @OA\Response(
     *         response=401,
     *         description="Unauthenticated",
     *         @OA\JsonContent(
     *             @OA\Property(property="message", type="string", example="Unauthenticated")
     *         )
     *     ),
     *     @OA\Response(
     *         response=403,
     *         description="Forbidden",
     *         @OA\JsonContent(
     *             @OA\Property(property="message", type="string", example="Unauthorized. Admin access required.")
     *         )
     *     )
     * )
     */
}
