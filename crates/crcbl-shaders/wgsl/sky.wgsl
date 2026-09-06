struct _MatrixStorage_float4x4_ColMajorstd140_0
{
    @align(16) data_0 : array<vec4<f32>, i32(4)>,
};

struct SkyParams_std140_0
{
    @align(16) inv_proj_0 : _MatrixStorage_float4x4_ColMajorstd140_0,
    @align(16) inv_view_0 : _MatrixStorage_float4x4_ColMajorstd140_0,
    @align(16) sky_0 : array<vec4<f32>, i32(3)>,
    @align(16) atmosphere_0 : vec4<f32>,
    @align(16) sun_disc_0 : vec4<f32>,
};

@binding(0) @group(0) var<uniform> camera_0 : SkyParams_std140_0;
@binding(1) @group(0) var<storage, read> sky_view_0 : array<vec4<f32>>;

var<private> SUN_LIMB_FIT_0 : array<vec3<f32>, i32(7)> = array<vec3<f32>, i32(7)>( vec3<f32>(-5.77344098928733729e-06f, 3.9952874431037344e-07f, -3.33302978106075898e-06f), vec3<f32>(0.00083442457253113f, -0.00004951301889378f, 0.00036370038287714f), vec3<f32>(-0.02268672175705433f, 0.00105253444053233f, -0.00656718015670776f), vec3<f32>(0.77790427207946777f, -0.01045561581850052f, 0.04684425517916679f), vec3<f32>(0.37402597069740295f, 0.98949694633483887f, -0.18627837300300598f), vec3<f32>(-0.17161113023757935f, 0.02444075979292393f, 1.0402073860168457f), vec3<f32>(0.04154470935463905f, -0.00448589120060205f, 0.10543691366910934f) );
struct FullscreenOutput_0
{
    @builtin(position) position_0 : vec4<f32>,
    @location(0) uv_0 : vec2<f32>,
};

@vertex
fn vertexMain(@builtin(vertex_index) index_0 : u32) -> FullscreenOutput_0
{
    var output_0 : FullscreenOutput_0;
    var _S1 : vec2<f32> = vec2<f32>(f32((((index_0 << (u32(1)))) & (u32(2)))), f32((index_0 & (u32(2)))));
    output_0.uv_0 = _S1;
    output_0.position_0 = vec4<f32>(_S1 * vec2<f32>(2.0f, -2.0f) + vec2<f32>(-1.0f, 1.0f), 0.0f, 1.0f);
    return output_0;
}

fn sky_view_at_0( up_0 : f32,  azimuth_cosine_0 : f32) -> vec3<f32>
{
    var u_0 : f32 = sqrt(max(0.0f, (1.0f - clamp(azimuth_cosine_0, -1.0f, 1.0f)) * 0.5f));
    var clamped_0 : f32 = clamp(up_0, -1.0f, 1.0f);
    var root_0 : f32 = sqrt(abs(clamped_0));
    var _S2 : f32;
    if(clamped_0 >= 0.0f)
    {
        _S2 = root_0;
    }
    else
    {
        _S2 = - root_0;
    }
    var across_0 : f32 = clamp(u_0, 0.0f, 1.0f) * 96.0f - 0.5f;
    var x0_0 : f32 = clamp(floor(across_0), 0.0f, 95.0f);
    var fx_0 : f32 = clamp(across_0 - x0_0, 0.0f, 1.0f);
    var down_0 : f32 = clamp(0.5f + 0.5f * _S2, 0.0f, 1.0f) * 64.0f - 0.5f;
    var y0_0 : f32 = clamp(floor(down_0), 0.0f, 63.0f);
    var fy_0 : f32 = clamp(down_0 - y0_0, 0.0f, 1.0f);
    var row0_0 : u32 = u32(y0_0) * u32(96);
    var row1_0 : u32 = u32(min(y0_0 + 1.0f, 63.0f)) * u32(96);
    var _S3 : u32 = u32(x0_0);
    var _S4 : vec3<f32> = vec3<f32>((1.0f - fx_0));
    var _S5 : u32 = u32(min(x0_0 + 1.0f, 95.0f));
    var _S6 : vec3<f32> = vec3<f32>(fx_0);
    return (sky_view_0[row0_0 + _S3].xyz * _S4 + sky_view_0[row0_0 + _S5].xyz * _S6) * vec3<f32>((1.0f - fy_0)) + (sky_view_0[row1_0 + _S3].xyz * _S4 + sky_view_0[row1_0 + _S5].xyz * _S6) * vec3<f32>(fy_0);
}

fn atmosphere_radiance_0( direction_0 : vec3<f32>) -> vec3<f32>
{
    var sun_0 : vec3<f32> = camera_0.atmosphere_0.xyz;
    var _S7 : f32 = direction_0.x;
    var _S8 : f32 = direction_0.z;
    var view_flat_0 : f32 = sqrt(_S7 * _S7 + _S8 * _S8);
    var _S9 : f32 = sun_0.x;
    var _S10 : f32 = sun_0.z;
    var sun_flat_0 : f32 = sqrt(_S9 * _S9 + _S10 * _S10);
    var _S11 : bool;
    if(view_flat_0 > 0.0f)
    {
        _S11 = sun_flat_0 > 0.0f;
    }
    else
    {
        _S11 = false;
    }
    var cosine_0 : f32;
    if(_S11)
    {
        cosine_0 = (_S7 * _S9 + _S8 * _S10) / (view_flat_0 * sun_flat_0);
    }
    else
    {
        cosine_0 = 1.0f;
    }
    return sky_view_at_0(direction_0.y, cosine_0);
}

fn sun_disc_1( direction_1 : vec3<f32>) -> vec3<f32>
{
    var apart_0 : vec3<f32> = direction_1 - camera_0.atmosphere_0.xyz;
    var radius_squared_0 : f32 = 0.5f * dot(apart_0, apart_0) / camera_0.sun_disc_0.w;
    if(radius_squared_0 >= 1.0f)
    {
        return vec3<f32>(0.0f, 0.0f, 0.0f);
    }
    var _S12 : f32 = sqrt(sqrt(sqrt(sqrt(1.0f - radius_squared_0))));
    var limb_0 : vec3<f32> = vec3<f32>(0.04154470935463905f, -0.00448589120060205f, 0.10543691366910934f);
    var power_0 : i32 = i32(5);
    for(;;)
    {
        if(power_0 >= i32(0))
        {
        }
        else
        {
            break;
        }
        var _S13 : vec3<f32> = limb_0 * vec3<f32>(_S12) + SUN_LIMB_FIT_0[power_0];
        var power_1 : i32 = power_0 - i32(1);
        limb_0 = _S13;
        power_0 = power_1;
    }
    return camera_0.sun_disc_0.xyz * max(limb_0, vec3<f32>(0.0f, 0.0f, 0.0f));
}

fn sky_radiance_0( direction_2 : vec3<f32>) -> vec3<f32>
{
    var up_1 : f32 = clamp(direction_2.y, -1.0f, 1.0f);
    var far_0 : vec3<f32>;
    if(up_1 >= 0.0f)
    {
        far_0 = camera_0.sky_0[i32(0)].xyz;
    }
    else
    {
        far_0 = camera_0.sky_0[i32(2)].xyz;
    }
    var u_1 : f32 = abs(up_1);
    var blend_0 : f32 = u_1 * u_1 * (3.0f - 2.0f * u_1);
    return camera_0.sky_0[i32(1)].xyz * vec3<f32>((1.0f - blend_0)) + far_0 * vec3<f32>(blend_0);
}

struct pixelOutput_0
{
    @location(0) output_1 : vec4<f32>,
};

struct pixelInput_0
{
    @location(0) uv_1 : vec2<f32>,
};

@fragment
fn fragmentMain( _S14 : pixelInput_0, @builtin(position) position_1 : vec4<f32>) -> pixelOutput_0
{
    var ndc_0 : vec2<f32> = vec2<f32>(_S14.uv_1.x * 2.0f - 1.0f, 1.0f - _S14.uv_1.y * 2.0f);
    var near_plane_0 : vec4<f32> = (((vec4<f32>(ndc_0, 1.0f, 1.0f)) * (mat4x4<f32>(camera_0.inv_proj_0.data_0[i32(0)][i32(0)], camera_0.inv_proj_0.data_0[i32(1)][i32(0)], camera_0.inv_proj_0.data_0[i32(2)][i32(0)], camera_0.inv_proj_0.data_0[i32(3)][i32(0)], camera_0.inv_proj_0.data_0[i32(0)][i32(1)], camera_0.inv_proj_0.data_0[i32(1)][i32(1)], camera_0.inv_proj_0.data_0[i32(2)][i32(1)], camera_0.inv_proj_0.data_0[i32(3)][i32(1)], camera_0.inv_proj_0.data_0[i32(0)][i32(2)], camera_0.inv_proj_0.data_0[i32(1)][i32(2)], camera_0.inv_proj_0.data_0[i32(2)][i32(2)], camera_0.inv_proj_0.data_0[i32(3)][i32(2)], camera_0.inv_proj_0.data_0[i32(0)][i32(3)], camera_0.inv_proj_0.data_0[i32(1)][i32(3)], camera_0.inv_proj_0.data_0[i32(2)][i32(3)], camera_0.inv_proj_0.data_0[i32(3)][i32(3)]))));
    var beyond_0 : vec4<f32> = (((vec4<f32>(ndc_0, 0.5f, 1.0f)) * (mat4x4<f32>(camera_0.inv_proj_0.data_0[i32(0)][i32(0)], camera_0.inv_proj_0.data_0[i32(1)][i32(0)], camera_0.inv_proj_0.data_0[i32(2)][i32(0)], camera_0.inv_proj_0.data_0[i32(3)][i32(0)], camera_0.inv_proj_0.data_0[i32(0)][i32(1)], camera_0.inv_proj_0.data_0[i32(1)][i32(1)], camera_0.inv_proj_0.data_0[i32(2)][i32(1)], camera_0.inv_proj_0.data_0[i32(3)][i32(1)], camera_0.inv_proj_0.data_0[i32(0)][i32(2)], camera_0.inv_proj_0.data_0[i32(1)][i32(2)], camera_0.inv_proj_0.data_0[i32(2)][i32(2)], camera_0.inv_proj_0.data_0[i32(3)][i32(2)], camera_0.inv_proj_0.data_0[i32(0)][i32(3)], camera_0.inv_proj_0.data_0[i32(1)][i32(3)], camera_0.inv_proj_0.data_0[i32(2)][i32(3)], camera_0.inv_proj_0.data_0[i32(3)][i32(3)]))));
    var direction_3 : vec3<f32> = normalize((((vec4<f32>(beyond_0.xyz / vec3<f32>(beyond_0.w) - near_plane_0.xyz / vec3<f32>(near_plane_0.w), 0.0f)) * (mat4x4<f32>(camera_0.inv_view_0.data_0[i32(0)][i32(0)], camera_0.inv_view_0.data_0[i32(1)][i32(0)], camera_0.inv_view_0.data_0[i32(2)][i32(0)], camera_0.inv_view_0.data_0[i32(3)][i32(0)], camera_0.inv_view_0.data_0[i32(0)][i32(1)], camera_0.inv_view_0.data_0[i32(1)][i32(1)], camera_0.inv_view_0.data_0[i32(2)][i32(1)], camera_0.inv_view_0.data_0[i32(3)][i32(1)], camera_0.inv_view_0.data_0[i32(0)][i32(2)], camera_0.inv_view_0.data_0[i32(1)][i32(2)], camera_0.inv_view_0.data_0[i32(2)][i32(2)], camera_0.inv_view_0.data_0[i32(3)][i32(2)], camera_0.inv_view_0.data_0[i32(0)][i32(3)], camera_0.inv_view_0.data_0[i32(1)][i32(3)], camera_0.inv_view_0.data_0[i32(2)][i32(3)], camera_0.inv_view_0.data_0[i32(3)][i32(3)])))).xyz);
    var radiance_0 : vec3<f32>;
    if((camera_0.atmosphere_0.w) > 0.0f)
    {
        radiance_0 = atmosphere_radiance_0(direction_3) + sun_disc_1(direction_3);
    }
    else
    {
        radiance_0 = sky_radiance_0(direction_3);
    }
    var _S15 : pixelOutput_0 = pixelOutput_0( vec4<f32>(min(radiance_0, vec3<f32>(65504.0f, 65504.0f, 65504.0f)), 1.0f) );
    return _S15;
}

