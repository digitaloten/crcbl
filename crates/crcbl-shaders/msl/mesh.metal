#include <metal_stdlib>
#include <metal_math>
#include <metal_texture>
using namespace metal;

#line 2886 "shaders/mesh.slang"
constant array<float, int(5)> FOG_RATIO_KERNEL_0 = { 1.0f, 0.5f, 0.1666666716337204f, 0.0416666679084301f, 0.00833333376795053f };

#line 2881
constant array<float, int(8)> FOG_KERNEL_0 = { 1.0f, 1.0f, 0.5f, 0.1666666716337204f, 0.0416666679084301f, 0.00833333376795053f, 0.00138888892251998f, 0.0001984127011383f };

#line 3883
constant array<float3, int(2)> CASCADE_TINTS_0 = { float3(1.0f, 0.34999999403953552f, 0.34999999403953552f), float3(0.34999999403953552f, 0.55000001192092896f, 1.0f) };

#line 3366
constant array<float2, int(16)> SHADOW_SEARCH_DISC_0 = { float2(0.17677700519561768f, 0.0f), float2(-0.22577199339866638f, 0.20682600140571594f), float2(0.0345579981803894f, -0.39377099275588989f), float2(0.28457099199295044f, 0.37117299437522888f), float2(-0.52222299575805664f, -0.09237399697303772f), float2(0.49469500780105591f, -0.31468498706817627f), float2(-0.16546599566936493f, 0.6155250072479248f), float2(-0.31556099653244019f, -0.60759401321411133f), float2(0.68464201688766479f, 0.25003001093864441f), float2(-0.71225601434707642f, 0.2940090000629425f), float2(0.3433539867401123f, -0.73372900485992432f), float2(0.25372999906539917f, 0.80893200635910034f), float2(-0.76474601030349731f, -0.44318601489067078f), float2(0.89713400602340698f, -0.19723199307918549f), float2(-0.54750698804855347f, 0.77877199649810791f), float2(-0.12648700177669525f, -0.97609001398086548f) };

#line 3153
constant array<float2, int(32)> SHADOW_DISC_0 = { float2(0.125f, 0.0f), float2(-0.15964500606060028f, 0.14624799787998199f), float2(0.02443600073456764f, -0.27843800187110901f), float2(0.2012220025062561f, 0.26245900988578796f), float2(-0.36926800012588501f, -0.06531800329685211f), float2(0.34980198740959167f, -0.22251600027084351f), float2(-0.11700200289487839f, 0.43524199724197388f), float2(-0.22313599288463593f, -0.42963400483131409f), float2(0.48411500453948975f, 0.17679800093173981f), float2(-0.50364100933074951f, 0.20789599418640137f), float2(0.24278800189495087f, -0.51882398128509521f), float2(0.17941400408744812f, 0.57200098037719727f), float2(-0.54075700044631958f, -0.31338000297546387f), float2(0.63437002897262573f, -0.13946400582790375f), float2(-0.38714599609375f, 0.55067497491836548f), float2(-0.0894400030374527f, -0.69019997119903564f), float2(0.5490720272064209f, 0.46275800466537476f), float2(-0.73887801170349121f, 0.0305550005286932f), float2(0.5389549732208252f, -0.53633201122283936f), float2(-0.03605800122022629f, 0.77979201078414917f), float2(-0.51281797885894775f, -0.61452698707580566f), float2(0.81235998868942261f, 0.10930199921131134f), float2(-0.68831098079681396f, 0.47890898585319519f), float2(0.18808600306510925f, -0.83606100082397461f), float2(0.43503299355506897f, 0.75919097661972046f), float2(-0.85044801235198975f, -0.27131599187850952f), float2(0.82610201835632324f, -0.38168001174926758f), float2(-0.35788801312446594f, 0.85515600442886353f), float2(-0.31940698623657227f, -0.88803398609161377f), float2(0.84990900754928589f, 0.44668799638748169f), float2(-0.94403499364852905f, 0.24884499609470367f), float2(0.53659600019454956f, -0.83452999591827393f) };

#line 3213
constant array<uint, int(5)> SHADOW_PROBE_INDEX_0 = { 0U, 23U, 25U, 27U, 29U };

#line 3228
constant array<float2, int(16)> SHADOW_ROTATIONS_0 = { float2(1.0f, 0.0f), float2(0.92387998104095459f, 0.38268300890922546f), float2(0.70710700750350952f, 0.70710700750350952f), float2(0.38268300890922546f, 0.92387998104095459f), float2(0.0f, 1.0f), float2(-0.38268300890922546f, 0.92387998104095459f), float2(-0.70710700750350952f, 0.70710700750350952f), float2(-0.92387998104095459f, 0.38268300890922546f), float2(-1.0f, 0.0f), float2(-0.92387998104095459f, -0.38268300890922546f), float2(-0.70710700750350952f, -0.70710700750350952f), float2(-0.38268300890922546f, -0.92387998104095459f), float2(-0.0f, -1.0f), float2(0.38268300890922546f, -0.92387998104095459f), float2(0.70710700750350952f, -0.70710700750350952f), float2(0.92387998104095459f, -0.38268300890922546f) };

#line 3256
constant array<uint, int(16)> SHADOW_DITHER_0 = { 0U, 8U, 2U, 10U, 12U, 4U, 14U, 6U, 3U, 11U, 1U, 9U, 15U, 7U, 13U, 5U };

#line 1329
struct DrawConstants_0
{
    uint base_0;
    uint mesh_0;
    uint pad0_0;
    uint pad1_0;
};


#line 2131
struct _MatrixStorage_float4x4_ColMajornatural_0
{
    array<packed_float4, int(4)> data_0;
};


#line 2131
struct GpuInstance_natural_0
{
    _MatrixStorage_float4x4_ColMajornatural_0 transform_0;
    _MatrixStorage_float4x4_ColMajornatural_0 previous_transform_0;
    uint mesh_1;
    uint material_0;
    uint sector_0;
    uint flags_0;
    uint base_vertex_0;
    uint previous_base_vertex_0;
    uint pad1_1;
    uint pad2_0;
};


#line 874
struct GpuMesh_0
{
    uint base_vertex_1;
    uint base_index_0;
    uint index_count_0;
    float min_x_0;
    float min_y_0;
    float min_z_0;
    float max_x_0;
    float max_y_0;
    float max_z_0;
    float uv_scale_u_0;
    float uv_scale_v_0;
    float uv_offset_u_0;
    float uv_offset_v_0;
    uint flags_1;
};


#line 2137
struct _MatrixStorage_float4x4_ColMajornatural_1
{
    array<float4, int(4)> data_1;
};


#line 2137
struct _Array_natural_matrixx3Cfloatx2C4x2C4x3E2_0
{
    array<_MatrixStorage_float4x4_ColMajornatural_1, int(2)> data_2;
};


#line 3332 "core.meta.slang"
struct _Array_natural_matrixx3Cfloatx2C4x2C4x3E14_0
{
    array<_MatrixStorage_float4x4_ColMajornatural_1, int(14)> data_3;
};


#line 363 "shaders/mesh.slang"
struct FrameUniforms_natural_0
{
    _MatrixStorage_float4x4_ColMajornatural_1 view_proj_0;
    float4 camera_position_0;
    float4 ambient_0;
    _Array_natural_matrixx3Cfloatx2C4x2C4x3E2_0 shadow_view_proj_0;
    float4 cascade_far_0;
    float4 shadow_params_0;
    uint4 cluster_grid_0;
    _Array_natural_matrixx3Cfloatx2C4x2C4x3E14_0 light_view_proj_0;
    uint4 probe_counts_0;
    uint4 probe_levels_0;
    array<float4, int(4)> probe_level_origin_0;
    array<float4, int(4)> probe_level_inv_spacing_0;
    array<uint4, int(4)> probe_level_offset_0;
    float4 lod_params_0;
    float4 fog_params_0;
    float4 fog_color_0;
    float4 sky_sh_r_0;
    float4 sky_sh_g_0;
    float4 sky_sh_b_0;
    _MatrixStorage_float4x4_ColMajornatural_1 previous_view_proj_0;
    uint4 vertex_pool_0;
    array<float4, int(16)> shadow_atlas_rect_0;
    uint4 shadow_filter_0;
};


#line 363
struct GpuMaterial_natural_0
{
    packed_float4 base_color_0;
    uint color_normal_pages_0;
    float metallic_0;
    float roughness_0;
    uint tiling_0;
    float tile_metres_0;
    float emissive_r_0;
    float emissive_g_0;
    float emissive_b_0;
    uint mro_emissive_pages_0;
    float normal_scale_0;
    float alpha_cutoff_0;
    uint flags_2;
};


#line 363
struct GpuLight_natural_0
{
    packed_float4 position_0;
    packed_float4 color_0;
    packed_float4 direction_0;
    packed_float4 tangent_0;
    uint kind_0;
    float cos_inner_0;
    uint shadow_tile_0;
    uint flags_3;
};


#line 363
struct GpuProbe_natural_0
{
    packed_float4 sh_r_0;
    packed_float4 sh_g_0;
    packed_float4 sh_b_0;
};


#line 363
struct KernelContext_0
{
    DrawConstants_0 constant* draw_0;
    uint device* visible_instances_0;
    GpuInstance_natural_0 device* instances_0;
    GpuMesh_0 device* meshes_0;
    FrameUniforms_natural_0 constant* frame_0;
    uint device* vertices_0;
    texture2d<float, access::sample> ambient_occlusion_0;
    GpuMaterial_natural_0 device* materials_0;
    texture2d_array<float, access::sample> base_color_textures_0;
    sampler base_color_sampler_0;
    texture2d_array<float, access::sample> normal_textures_0;
    texture2d_array<float, access::sample> mro_textures_0;
    texture2d_array<float, access::sample> emissive_textures_0;
    uint device* cluster_lights_0;
    texture2d<float, access::sample> specular_dfg_0;
    GpuLight_natural_0 device* lights_0;
    texture2d<float, access::sample> ltc_matrix_0;
    depth2d<float, access::sample> shadow_atlas_0;
    sampler shadow_sampler_0;
    texture2d<float, access::sample> contact_shadow_0;
    GpuProbe_natural_0 device* probes_0;
    texture2d_array<float, access::sample> probe_visibility_0;
};


#line 1372
float3 load_position_0(uint at_0, KernelContext_0 thread* kernelContext_0)
{
    uint word_0 = at_0 * 3U;
    return float3((as_type<float>((kernelContext_0->vertices_0[word_0]))), (as_type<float>((kernelContext_0->vertices_0[word_0 + 1U]))), (as_type<float>((kernelContext_0->vertices_0[word_0 + 2U]))));
}


#line 196
float dequantise_snorm_0(int lane_0)
{
    return max(float(lane_0) / 32767.0f, -1.0f);
}


float4 unpack_snorm16x4_0(uint low_0, uint high_0)
{
    return float4(dequantise_snorm_0((as_type<int>((low_0 << 16U))) >> 16U), dequantise_snorm_0((as_type<int>((low_0))) >> 16U), dequantise_snorm_0((as_type<int>((high_0 << 16U))) >> 16U), dequantise_snorm_0((as_type<int>((high_0))) >> 16U));
}


#line 228
float3 rotate_by_0(float4 q_0, float3 v_0)
{
    float3 _S1 = q_0.xyz;

#line 230
    float3 t_0 = float3(2.0f)  * cross(_S1, v_0);
    return v_0 + float3(q_0.w)  * t_0 + cross(_S1, t_0);
}


#line 186
struct TangentFrame_0
{
    float3 tangent_1;
    float3 bitangent_0;
    float3 normal_0;
};


#line 242
TangentFrame_0 decode_qtangent_0(float4 lanes_0)
{
    float4 q_1 = normalize(lanes_0);
    thread TangentFrame_0 basis_0;
    float3 _S2 = rotate_by_0(q_1, float3(1.0f, 0.0f, 0.0f));

#line 246
    (&basis_0)->tangent_1 = _S2;
    float3 _S3 = rotate_by_0(q_1, float3(0.0f, 0.0f, 1.0f));

#line 247
    (&basis_0)->normal_0 = _S3;
    float3 _S4 = cross(_S3, _S2);

#line 248
    float _S5;

#line 248
    if((lanes_0.w) < 0.0f)
    {

#line 248
        _S5 = -1.0f;

#line 248
    }
    else
    {

#line 248
        _S5 = 1.0f;

#line 248
    }

#line 248
    (&basis_0)->bitangent_0 = _S4 * float3(_S5) ;
    return basis_0;
}


#line 211
float2 unpack_unorm16x2_0(uint word_1)
{
    return float2(float(word_1 & 65535U), float(word_1 >> 16U)) / float2(65535.0f) ;
}


float4 unpack_rgba8_0(uint word_2)
{
    return float4(float(word_2 & 255U), float((word_2 >> 8U) & 255U), float((word_2 >> 16U) & 255U), float(word_2 >> 24U)) / float4(255.0f) ;
}


#line 257
struct MeshVertex_0
{
    float3 position_1;
    TangentFrame_0 basis_1;
    float2 uv0_0;
    float4 color_1;
};


#line 1383
MeshVertex_0 load_vertex_0(uint at_1, float4 range_0, KernelContext_0 thread* kernelContext_1)
{
    uint word_3 = kernelContext_1->frame_0->vertex_pool_0.x + at_1 * 5U;
    thread MeshVertex_0 vertex_0;

#line 1386
    float3 _S6 = load_position_0(at_1, kernelContext_1);
    (&vertex_0)->position_1 = _S6;
    (&vertex_0)->basis_1 = decode_qtangent_0(unpack_snorm16x4_0(kernelContext_1->vertices_0[word_3], kernelContext_1->vertices_0[word_3 + 1U]));
    (&vertex_0)->uv0_0 = range_0.zw + range_0.xy * unpack_unorm16x2_0(kernelContext_1->vertices_0[word_3 + 2U]);
    (&vertex_0)->color_1 = unpack_rgba8_0(kernelContext_1->vertices_0[word_3 + 4U]);
    return vertex_0;
}


#line 1985
matrix<float,int(3),int(3)>  normal_basis_0(matrix<float,int(3),int(3)>  basis_2)
{
    return matrix<float,int(3),int(3)> (cross(basis_2[int(1)], basis_2[int(2)]), cross(basis_2[int(2)], basis_2[int(0)]), cross(basis_2[int(0)], basis_2[int(1)]));
}


#line 2118
uint frame_word_0(uint mesh_flags_0, const TangentFrame_0 thread* basis_3)
{

#line 2118
    uint word_4;

    if((mesh_flags_0 & 1U) != 0U)
    {

#line 2120
        word_4 = 1U;

#line 2120
    }
    else
    {

#line 2120
        word_4 = 0U;

#line 2120
    }



    if((dot(cross(basis_3->normal_0, basis_3->tangent_1), basis_3->bitangent_0)) < 0.0f)
    {

#line 2124
        word_4 = word_4 | 2U;

#line 2124
    }

#line 2123
    return word_4;
}


#line 2123
struct vertexOutput_0
{
    float4 output_0 [[position]];
};


#line 2239
[[vertex]] vertexOutput_0 depthVertexMain(uint index_0 [[vertex_id]], uint instance_id_0 [[instance_id]], DrawConstants_0 constant* draw_1 [[buffer(3)]], uint device* visible_instances_1 [[buffer(5)]], GpuInstance_natural_0 device* instances_1 [[buffer(2)]], GpuMesh_0 device* meshes_1 [[buffer(4)]], FrameUniforms_natural_0 constant* frame_1 [[buffer(0)]], uint device* vertices_1 [[buffer(1)]], texture2d<float, access::sample> ambient_occlusion_1 [[texture(2)]], GpuMaterial_natural_0 device* materials_1 [[buffer(6)]], texture2d_array<float, access::sample> base_color_textures_1 [[texture(0)]], sampler base_color_sampler_1 [[sampler(0)]], texture2d_array<float, access::sample> normal_textures_1 [[texture(4)]], texture2d_array<float, access::sample> mro_textures_1 [[texture(8)]], texture2d_array<float, access::sample> emissive_textures_1 [[texture(9)]], uint device* cluster_lights_1 [[buffer(8)]], texture2d<float, access::sample> specular_dfg_1 [[texture(3)]], GpuLight_natural_0 device* lights_1 [[buffer(7)]], texture2d<float, access::sample> ltc_matrix_1 [[texture(5)]], depth2d<float, access::sample> shadow_atlas_1 [[texture(1)]], sampler shadow_sampler_1 [[sampler(1)]], texture2d<float, access::sample> contact_shadow_1 [[texture(6)]], GpuProbe_natural_0 device* probes_1 [[buffer(9)]], texture2d_array<float, access::sample> probe_visibility_1 [[texture(7)]])
{

#line 2239
    thread KernelContext_0 kernelContext_2;

#line 2239
    (&kernelContext_2)->draw_0 = draw_1;

#line 2239
    (&kernelContext_2)->visible_instances_0 = visible_instances_1;

#line 2239
    (&kernelContext_2)->instances_0 = instances_1;

#line 2239
    (&kernelContext_2)->meshes_0 = meshes_1;

#line 2239
    (&kernelContext_2)->frame_0 = frame_1;

#line 2239
    (&kernelContext_2)->vertices_0 = vertices_1;

#line 2239
    (&kernelContext_2)->ambient_occlusion_0 = ambient_occlusion_1;

#line 2239
    (&kernelContext_2)->materials_0 = materials_1;

#line 2239
    (&kernelContext_2)->base_color_textures_0 = base_color_textures_1;

#line 2239
    (&kernelContext_2)->base_color_sampler_0 = base_color_sampler_1;

#line 2239
    (&kernelContext_2)->normal_textures_0 = normal_textures_1;

#line 2239
    (&kernelContext_2)->mro_textures_0 = mro_textures_1;

#line 2239
    (&kernelContext_2)->emissive_textures_0 = emissive_textures_1;

#line 2239
    (&kernelContext_2)->cluster_lights_0 = cluster_lights_1;

#line 2239
    (&kernelContext_2)->specular_dfg_0 = specular_dfg_1;

#line 2239
    (&kernelContext_2)->lights_0 = lights_1;

#line 2239
    (&kernelContext_2)->ltc_matrix_0 = ltc_matrix_1;

#line 2239
    (&kernelContext_2)->shadow_atlas_0 = shadow_atlas_1;

#line 2239
    (&kernelContext_2)->shadow_sampler_0 = shadow_sampler_1;

#line 2239
    (&kernelContext_2)->contact_shadow_0 = contact_shadow_1;

#line 2239
    (&kernelContext_2)->probes_0 = probes_1;

#line 2239
    (&kernelContext_2)->probe_visibility_0 = probe_visibility_1;

#line 2239
    GpuInstance_natural_0 device* _S7 = instances_1+visible_instances_1[draw_1->base_0 + instance_id_0];


    GpuMesh_0 mesh_2 = meshes_1[draw_1->mesh_0];

#line 2242
    uint base_vertex_2;

#line 2248
    if(((_S7->flags_0) & 2U) != 0U)
    {

#line 2248
        base_vertex_2 = _S7->base_vertex_0;

#line 2248
    }
    else
    {

#line 2248
        base_vertex_2 = mesh_2.base_vertex_1;

#line 2248
    }

#line 2248
    matrix<float,int(4),int(4)>  _S8 = matrix<float,int(4),int(4)> (_S7->transform_0.data_0[int(0)][int(0)], _S7->transform_0.data_0[int(1)][int(0)], _S7->transform_0.data_0[int(2)][int(0)], _S7->transform_0.data_0[int(3)][int(0)], _S7->transform_0.data_0[int(0)][int(1)], _S7->transform_0.data_0[int(1)][int(1)], _S7->transform_0.data_0[int(2)][int(1)], _S7->transform_0.data_0[int(3)][int(1)], _S7->transform_0.data_0[int(0)][int(2)], _S7->transform_0.data_0[int(1)][int(2)], _S7->transform_0.data_0[int(2)][int(2)], _S7->transform_0.data_0[int(3)][int(2)], _S7->transform_0.data_0[int(0)][int(3)], _S7->transform_0.data_0[int(1)][int(3)], _S7->transform_0.data_0[int(2)][int(3)], _S7->transform_0.data_0[int(3)][int(3)]);

#line 2248
    float3 _S9 = load_position_0(index_0 + base_vertex_2, &kernelContext_2);

#line 2248
    vertexOutput_0 _S10 = { ((((((float4(_S9, 1.0f)) * (_S8)))) * (matrix<float,int(4),int(4)> ((&kernelContext_2)->frame_0->view_proj_0.data_1[int(0)][int(0)], (&kernelContext_2)->frame_0->view_proj_0.data_1[int(1)][int(0)], (&kernelContext_2)->frame_0->view_proj_0.data_1[int(2)][int(0)], (&kernelContext_2)->frame_0->view_proj_0.data_1[int(3)][int(0)], (&kernelContext_2)->frame_0->view_proj_0.data_1[int(0)][int(1)], (&kernelContext_2)->frame_0->view_proj_0.data_1[int(1)][int(1)], (&kernelContext_2)->frame_0->view_proj_0.data_1[int(2)][int(1)], (&kernelContext_2)->frame_0->view_proj_0.data_1[int(3)][int(1)], (&kernelContext_2)->frame_0->view_proj_0.data_1[int(0)][int(2)], (&kernelContext_2)->frame_0->view_proj_0.data_1[int(1)][int(2)], (&kernelContext_2)->frame_0->view_proj_0.data_1[int(2)][int(2)], (&kernelContext_2)->frame_0->view_proj_0.data_1[int(3)][int(2)], (&kernelContext_2)->frame_0->view_proj_0.data_1[int(0)][int(3)], (&kernelContext_2)->frame_0->view_proj_0.data_1[int(1)][int(3)], (&kernelContext_2)->frame_0->view_proj_0.data_1[int(2)][int(3)], (&kernelContext_2)->frame_0->view_proj_0.data_1[int(3)][int(3)])))) };


    return _S10;
}


#line 2251
struct vertexOutput_1
{
    float4 output_1 [[position]];
};


#line 2272
[[vertex]] vertexOutput_1 depthClearVertexMain(uint index_1 [[vertex_id]], DrawConstants_0 constant* draw_2 [[buffer(3)]], uint device* visible_instances_2 [[buffer(5)]], GpuInstance_natural_0 device* instances_2 [[buffer(2)]], GpuMesh_0 device* meshes_2 [[buffer(4)]], FrameUniforms_natural_0 constant* frame_2 [[buffer(0)]], uint device* vertices_2 [[buffer(1)]], texture2d<float, access::sample> ambient_occlusion_2 [[texture(2)]], GpuMaterial_natural_0 device* materials_2 [[buffer(6)]], texture2d_array<float, access::sample> base_color_textures_2 [[texture(0)]], sampler base_color_sampler_2 [[sampler(0)]], texture2d_array<float, access::sample> normal_textures_2 [[texture(4)]], texture2d_array<float, access::sample> mro_textures_2 [[texture(8)]], texture2d_array<float, access::sample> emissive_textures_2 [[texture(9)]], uint device* cluster_lights_2 [[buffer(8)]], texture2d<float, access::sample> specular_dfg_2 [[texture(3)]], GpuLight_natural_0 device* lights_2 [[buffer(7)]], texture2d<float, access::sample> ltc_matrix_2 [[texture(5)]], depth2d<float, access::sample> shadow_atlas_2 [[texture(1)]], sampler shadow_sampler_2 [[sampler(1)]], texture2d<float, access::sample> contact_shadow_2 [[texture(6)]], GpuProbe_natural_0 device* probes_2 [[buffer(9)]], texture2d_array<float, access::sample> probe_visibility_2 [[texture(7)]])
{

#line 2272
    thread KernelContext_0 kernelContext_3;

#line 2272
    (&kernelContext_3)->draw_0 = draw_2;

#line 2272
    (&kernelContext_3)->visible_instances_0 = visible_instances_2;

#line 2272
    (&kernelContext_3)->instances_0 = instances_2;

#line 2272
    (&kernelContext_3)->meshes_0 = meshes_2;

#line 2272
    (&kernelContext_3)->frame_0 = frame_2;

#line 2272
    (&kernelContext_3)->vertices_0 = vertices_2;

#line 2272
    (&kernelContext_3)->ambient_occlusion_0 = ambient_occlusion_2;

#line 2272
    (&kernelContext_3)->materials_0 = materials_2;

#line 2272
    (&kernelContext_3)->base_color_textures_0 = base_color_textures_2;

#line 2272
    (&kernelContext_3)->base_color_sampler_0 = base_color_sampler_2;

#line 2272
    (&kernelContext_3)->normal_textures_0 = normal_textures_2;

#line 2272
    (&kernelContext_3)->mro_textures_0 = mro_textures_2;

#line 2272
    (&kernelContext_3)->emissive_textures_0 = emissive_textures_2;

#line 2272
    (&kernelContext_3)->cluster_lights_0 = cluster_lights_2;

#line 2272
    (&kernelContext_3)->specular_dfg_0 = specular_dfg_2;

#line 2272
    (&kernelContext_3)->lights_0 = lights_2;

#line 2272
    (&kernelContext_3)->ltc_matrix_0 = ltc_matrix_2;

#line 2272
    (&kernelContext_3)->shadow_atlas_0 = shadow_atlas_2;

#line 2272
    (&kernelContext_3)->shadow_sampler_0 = shadow_sampler_2;

#line 2272
    (&kernelContext_3)->contact_shadow_0 = contact_shadow_2;

#line 2272
    (&kernelContext_3)->probes_0 = probes_2;

#line 2272
    (&kernelContext_3)->probe_visibility_0 = probe_visibility_2;

#line 2272
    vertexOutput_1 _S11 = { float4(float2(float((index_1 << 1U) & 2U), float(index_1 & 2U)) * float2(2.0f, -2.0f) + float2(-1.0f, 1.0f), 0.0f, 1.0f) };


    return _S11;
}


#line 5288
float2 motion_vector_0(float4 current_0, float4 previous_0)
{
    float _S12 = previous_0.w;

#line 5290
    if(_S12 <= 0.0f)
    {
        return float2(0.0f, 0.0f);
    }
    return (current_0.xy / float2(current_0.w)  - previous_0.xy / float2(_S12) ) * float2(0.5f, -0.5f);
}


#line 5164
float4 occlusion_at_0(float2 position_2, KernelContext_0 thread* kernelContext_4)
{

#line 5164
    texture2d<float, access::sample> _S13 = kernelContext_4->ambient_occlusion_0;

    thread uint width_0;
    thread uint height_0;
    (*((&width_0)) = (_S13).get_width(0)),(*((&height_0)) = (_S13).get_height(0));

    int3 _S14 = int3(min(int2(position_2), int2(int(width_0), int(height_0)) - int2(int(1)) ), int(0));

#line 5170
    return ((kernelContext_4->ambient_occlusion_0).read(vec<uint,2>(((_S14)).xy), uint(((_S14)).z)));
}


#line 4898
float2 physical_tile_uv_0(float3 world_position_0, float3 normal_1, float tile_metres_1)
{
    float3 axis_0 = abs(normal_1);

    float _S15 = axis_0.x;

#line 4902
    float _S16 = axis_0.y;

#line 4902
    bool _S17;

#line 4902
    if(_S15 >= _S16)
    {

#line 4902
        _S17 = _S15 >= (axis_0.z);

#line 4902
    }
    else
    {

#line 4902
        _S17 = false;

#line 4902
    }

#line 4902
    float2 planar_0;

#line 4902
    if(_S17)
    {

#line 4902
        planar_0 = world_position_0.zy;

#line 4902
    }
    else
    {

        if(_S16 >= (axis_0.z))
        {

#line 4906
            planar_0 = world_position_0.xz;

#line 4906
        }
        else
        {

#line 4906
            planar_0 = world_position_0.xy;

#line 4906
        }

#line 4902
    }

#line 4914
    return planar_0 / float2(max(tile_metres_1, 0.00009999999747379f)) ;
}


#line 1060
uint base_color_layer_0(const GpuMaterial_natural_0 thread* material_1)
{
    return (material_1->color_normal_pages_0) & 65535U;
}


#line 1488
float4 base_color_texel_0(const GpuMaterial_natural_0 thread* material_2, float2 uv_0, KernelContext_0 thread* kernelContext_5)
{
    float2 duvdx_0 = dfdx(uv_0);
    float2 duvdy_0 = dfdy(uv_0);

#line 1491
    uint _S18 = base_color_layer_0(material_2);

    if(_S18 == 65535U)
    {
        return float4(1.0f, 1.0f, 1.0f, 1.0f);
    }

    float3 _S19 = float3(uv_0, float(_S18));

#line 1497
    return ((kernelContext_5->base_color_textures_0).sample((kernelContext_5->base_color_sampler_0), ((_S19)).xy, uint(((_S19)).z), gradient2d((duvdx_0), (duvdy_0))));
}


#line 1169
bool alpha_masked_0(const GpuMaterial_natural_0 thread* material_3, float alpha_0)
{

#line 1169
    bool _S20;

    if(((material_3->flags_2) & 1U) != 0U)
    {

#line 1171
        _S20 = alpha_0 < (material_3->alpha_cutoff_0);

#line 1171
    }
    else
    {

#line 1171
        _S20 = false;

#line 1171
    }

#line 1171
    return _S20;
}


#line 1204
float3 double_sided_normal_0(const GpuMaterial_natural_0 thread* material_4, float3 normal_2, bool front_facing_0)
{

#line 1204
    bool _S21;

    if(((material_4->flags_2) & 2U) != 0U)
    {

#line 1206
        _S21 = !front_facing_0;

#line 1206
    }
    else
    {

#line 1206
        _S21 = false;

#line 1206
    }

#line 1206
    float3 _S22;

#line 1206
    if(_S21)
    {

#line 1206
        _S22 = - normal_2;

#line 1206
    }
    else
    {

#line 1206
        _S22 = normal_2;

#line 1206
    }

#line 1206
    return _S22;
}


#line 1075
uint normal_layer_0(const GpuMaterial_natural_0 thread* material_5)
{
    return (material_5->color_normal_pages_0) >> 16U;
}


#line 4935
float3 orthonormal_tangent_0(float3 normal_3)
{
    float _S23 = normal_3.z;

#line 4937
    float sign_z_0;

#line 4937
    if(_S23 >= 0.0f)
    {

#line 4937
        sign_z_0 = 1.0f;

#line 4937
    }
    else
    {

#line 4937
        sign_z_0 = -1.0f;

#line 4937
    }
    float a_0 = -1.0f / (sign_z_0 + _S23);
    float _S24 = normal_3.x;

#line 4939
    float _S25 = sign_z_0 * _S24;

#line 4939
    return float3(1.0f + _S25 * _S24 * a_0, _S25 * normal_3.y * a_0, - sign_z_0 * _S24);
}


#line 4989
TangentFrame_0 derivative_frame_0(float3 dpdx_0, float3 dpdy_0, float2 duvdx_1, float2 duvdy_1, float3 normal_4)
{
    float _S26 = duvdy_1.y;

#line 4991
    float _S27 = duvdx_1.y;

#line 4991
    float winding_0;
    if((duvdx_1.x * _S26 - duvdy_1.x * _S27) < 0.0f)
    {

#line 4992
        winding_0 = -1.0f;

#line 4992
    }
    else
    {

#line 4992
        winding_0 = 1.0f;

#line 4992
    }
    float3 tangent_2 = (float3(_S26)  * dpdx_0 - float3(_S27)  * dpdy_0) * float3(winding_0) ;

    thread TangentFrame_0 basis_4;
    (&basis_4)->normal_0 = normal_4;

#line 5001
    float3 tangent_3 = tangent_2 - normal_4 * float3(dot(normal_4, tangent_2)) ;
    float length_squared_0 = dot(tangent_3, tangent_3);

#line 5002
    float3 _S28;

#line 5011
    if(length_squared_0 > 1.00000001686238353e-16f)
    {

#line 5011
        _S28 = tangent_3 * float3(rsqrt(length_squared_0)) ;

#line 5011
    }
    else
    {

#line 5011
        _S28 = orthonormal_tangent_0(normal_4);

#line 5011
    }

#line 5011
    (&basis_4)->tangent_1 = _S28;

    (&basis_4)->bitangent_0 = cross(normal_4, _S28);
    return basis_4;
}


#line 2002
struct VertexOutput_0
{
    float4 position_3;
    float3 world_position_1;
    float3 world_normal_0;
    float4 color_2;
    [[flat]] uint material_6;
    float2 uv_1;
    float4 clip_position_0;
    float4 previous_clip_position_0;
    float3 world_tangent_0;
    [[flat]] uint frame_3;
};


#line 5071
float3 shading_normal_of_0(uint layer_0, float normal_scale_1, const VertexOutput_0 thread* input_0, float3 normal_5, float2 uv_2, KernelContext_0 thread* kernelContext_6)
{

#line 5083
    float3 dpdx_1 = dfdx(input_0->world_position_1);
    float3 dpdy_1 = dfdy(input_0->world_position_1);
    float2 duvdx_2 = dfdx(uv_2);
    float2 duvdy_2 = dfdy(uv_2);

    if(layer_0 == 65535U)
    {
        return normal_5;
    }

    thread TangentFrame_0 basis_5;

#line 5093
    uint _S29 = input_0->frame_3;
    if(((input_0->frame_3) & 1U) != 0U)
    {

#line 5102
        (&basis_5)->normal_0 = normal_5;
        float3 tangent_4 = input_0->world_tangent_0 - normal_5 * float3(dot(normal_5, input_0->world_tangent_0)) ;
        float length_squared_1 = dot(tangent_4, tangent_4);

#line 5104
        float3 _S30;

#line 5109
        if(length_squared_1 > 1.00000001686238353e-16f)
        {

#line 5109
            _S30 = tangent_4 * float3(rsqrt(length_squared_1)) ;

#line 5109
        }
        else
        {

#line 5109
            _S30 = orthonormal_tangent_0(normal_5);

#line 5109
        }

#line 5109
        (&basis_5)->tangent_1 = _S30;

#line 5115
        float3 _S31 = cross((&basis_5)->normal_0, _S30);

#line 5115
        float _S32;
        if((_S29 & 2U) != 0U)
        {

#line 5116
            _S32 = -1.0f;

#line 5116
        }
        else
        {

#line 5116
            _S32 = 1.0f;

#line 5116
        }

#line 5115
        (&basis_5)->bitangent_0 = _S31 * float3(_S32) ;

#line 5094
    }
    else
    {

#line 5120
        basis_5 = derivative_frame_0(dpdx_1, dpdy_1, duvdx_2, duvdy_2, normal_5);

#line 5094
    }

#line 5124
    float3 _S33 = float3(uv_2, float(layer_0));
    float3 _S34 = ((kernelContext_6->normal_textures_0).sample((kernelContext_6->base_color_sampler_0), ((_S33)).xy, uint(((_S33)).z), gradient2d((duvdx_2), (duvdy_2)))).xyz * float3(2.0f)  - float3(1.0f) ;

#line 5125
    thread float3 tangent_space_0 = _S34;
    tangent_space_0.xy = _S34.xy * float2(normal_scale_1) ;

#line 5131
    float3 _S35 = normalize(tangent_space_0);

#line 5131
    tangent_space_0 = _S35;
    return normalize(float3(_S35.x)  * (&basis_5)->tangent_1 + float3(_S35.y)  * (&basis_5)->bitangent_0 + float3(_S35.z)  * (&basis_5)->normal_0);
}


#line 3021
float3 geometric_normal_of_0(float3 world_position_2, float3 shading_normal_0)
{
    float3 facet_0 = cross(dfdx(world_position_2), dfdy(world_position_2));
    float extent_0 = length(facet_0);
    if(extent_0 < 9.999999960041972e-13f)
    {



        return shading_normal_0;
    }
    float3 facet_1 = facet_0 / float3(extent_0) ;

#line 3032
    float3 _S36;
    if((dot(facet_1, shading_normal_0)) < 0.0f)
    {

#line 3033
        _S36 = - facet_1;

#line 3033
    }
    else
    {

#line 3033
        _S36 = facet_1;

#line 3033
    }

#line 3033
    return _S36;
}


#line 1093
uint mro_layer_0(const GpuMaterial_natural_0 thread* material_7)
{
    return (material_7->mro_emissive_pages_0) & 65535U;
}


#line 1892
float4 mro_texel_0(const GpuMaterial_natural_0 thread* material_8, float2 uv_3, KernelContext_0 thread* kernelContext_7)
{
    float2 duvdx_3 = dfdx(uv_3);
    float2 duvdy_3 = dfdy(uv_3);

#line 1895
    uint _S37 = mro_layer_0(material_8);

    if(_S37 == 65535U)
    {
        return float4(1.0f, 1.0f, 1.0f, 1.0f);
    }

    float3 _S38 = float3(uv_3, float(_S37));

#line 1901
    return ((kernelContext_7->mro_textures_0).sample((kernelContext_7->base_color_sampler_0), ((_S38)).xy, uint(((_S38)).z), gradient2d((duvdx_3), (duvdy_3))));
}


#line 1105
uint emissive_layer_0(const GpuMaterial_natural_0 thread* material_9)
{
    return (material_9->mro_emissive_pages_0) >> 16U;
}


#line 1911
float4 emissive_texel_0(const GpuMaterial_natural_0 thread* material_10, float2 uv_4, KernelContext_0 thread* kernelContext_8)
{
    float2 duvdx_4 = dfdx(uv_4);
    float2 duvdy_4 = dfdy(uv_4);

#line 1914
    uint _S39 = emissive_layer_0(material_10);

    if(_S39 == 65535U)
    {
        return float4(1.0f, 1.0f, 1.0f, 1.0f);
    }

    float3 _S40 = float3(uv_4, float(_S39));

#line 1920
    return ((kernelContext_8->emissive_textures_0).sample((kernelContext_8->base_color_sampler_0), ((_S40)).xy, uint(((_S40)).z), gradient2d((duvdx_4), (duvdy_4))));
}


#line 1938
float metallic_of_0(const GpuMaterial_natural_0 thread* material_11, float4 mro_0)
{
    return saturate(material_11->metallic_0 * mro_0.z);
}


#line 2426
float specular_aa_kernel_0(float3 normal_6)
{
    float3 dndx_0 = dfdx(normal_6);
    float3 dndy_0 = dfdy(normal_6);


    return min(2.0f * (0.25f * (dot(dndx_0, dndx_0) + dot(dndy_0, dndy_0))), 0.18000000715255737f);
}


#line 4320
uint froxel_of_0(float2 pixel_0, float depth_0, KernelContext_0 thread* kernelContext_9)
{
    uint _S41 = max(kernelContext_9->frame_0->cluster_grid_0.x, 1U);
    uint _S42 = max(kernelContext_9->frame_0->cluster_grid_0.y, 1U);
    uint _S43 = max(kernelContext_9->frame_0->cluster_grid_0.z, 1U);
    uint _S44 = max(kernelContext_9->frame_0->cluster_grid_0.w, 1U);

#line 4330
    uint _S45 = uint(pixel_0.x) / _S44;

#line 4330
    uint _S46 = min(_S45, _S41 - 1U);
    uint _S47 = uint(pixel_0.y) / _S44;

    float scale_0 = 24.0f / log2(10000.0f);

#line 4341
    return (uint(clamp(floor(log2(max(depth_0, 0.10000000149011612f)) * scale_0 + - scale_0 * log2(0.10000000149011612f)), 0.0f, float(_S43 - 1U))) * _S42 + min(_S47, _S42 - 1U)) * _S41 + _S46;
}


#line 2453
struct TableTap_0
{
    int2 lo_0;
    int2 hi_0;
    float2 weight_0;
};


#line 2474
TableTap_0 table_tap_0(float n_dot_v_0, float roughness_1, KernelContext_0 thread* kernelContext_10)
{

#line 2474
    texture2d<float, access::sample> _S48 = kernelContext_10->specular_dfg_0;

    thread uint width_1;
    thread uint height_1;
    (*((&width_1)) = (_S48).get_width(0)),(*((&height_1)) = (_S48).get_height(0));
    float2 extent_1 = float2(float(width_1), float(height_1));
    float2 scaled_0 = float2(saturate(n_dot_v_0), saturate(roughness_1)) * extent_1 - float2(0.5f) ;

#line 2480
    float2 _S49 = float2(1.0f) ;
    float2 _S50 = extent_1 - _S49;

#line 2481
    float2 low_1 = clamp(floor(scaled_0), float2(0.0f, 0.0f), _S50);
    float2 high_1 = min(low_1 + _S49, _S50);

    thread TableTap_0 tap_0;
    (&tap_0)->lo_0 = int2(low_1);
    (&tap_0)->hi_0 = int2(high_1);
    (&tap_0)->weight_0 = clamp(scaled_0 - low_1, float2(0.0f) , float2(1.0f) );
    return tap_0;
}


#line 2499
float2 decode_dfg_pair_0(float4 texel_0)
{
    return float2(texel_0.x * 65280.0f + texel_0.y * 255.0f, texel_0.z * 65280.0f + texel_0.w * 255.0f) / float2(65535.0f) ;
}


#line 2511
float2 dfg_at_0(const TableTap_0 thread* tap_1, KernelContext_0 thread* kernelContext_11)
{
    int _S51 = tap_1->lo_0.x;

#line 2513
    int _S52 = tap_1->lo_0.y;

#line 2513
    int3 _S53 = int3(_S51, _S52, int(0));
    int _S54 = tap_1->hi_0.x;

#line 2514
    int3 _S55 = int3(_S54, _S52, int(0));
    float2 _S56 = float2(tap_1->weight_0.x) ;
    int _S57 = tap_1->hi_0.y;

#line 2516
    int3 _S58 = int3(_S51, _S57, int(0));
    int3 _S59 = int3(_S54, _S57, int(0));

    return mix(mix(decode_dfg_pair_0(((kernelContext_11->specular_dfg_0).read(vec<uint,2>(((_S53)).xy), uint(((_S53)).z)))), decode_dfg_pair_0(((kernelContext_11->specular_dfg_0).read(vec<uint,2>(((_S55)).xy), uint(((_S55)).z)))), _S56), mix(decode_dfg_pair_0(((kernelContext_11->specular_dfg_0).read(vec<uint,2>(((_S58)).xy), uint(((_S58)).z)))), decode_dfg_pair_0(((kernelContext_11->specular_dfg_0).read(vec<uint,2>(((_S59)).xy), uint(((_S59)).z)))), _S56), float2(tap_1->weight_0.y) );
}


#line 4271
float range_window_0(float distance_0, float radius_0)
{
    float ratio_0 = distance_0 / max(radius_0, 9.99999997475242708e-07f);
    float window_0 = saturate(1.0f - ratio_0 * ratio_0 * ratio_0 * ratio_0);
    return window_0 * window_0;
}


#line 4287
float punctual_falloff_0(float distance_1, float radius_1)
{
    return range_window_0(distance_1, radius_1) / (distance_1 * distance_1 + 1.0f);
}


#line 4299
float spot_cone_0(float3 to_light_0, float3 axis_1, float cos_outer_0, float cos_inner_1)
{

#line 4306
    return saturate((dot(- to_light_0, normalize(axis_1)) - cos_outer_0) / max(cos_inner_1 - cos_outer_0, 0.00009999999747379f));
}


#line 2840
void rect_corners_0(const GpuLight_natural_0 thread* light_0, float3 world_position_3, array<float3, int(4)> thread* corners_0)
{

#line 2840
    float4 _S60 = float4(light_0->tangent_0) ;

    float3 _S61 = _S60.xyz;

#line 2842
    float3 across_0 = _S61 * float3(_S60.w) ;

#line 2842
    float4 _S62 = float4(light_0->direction_0) ;
    float3 down_0 = cross(_S61, _S62.xyz) * float3(_S62.w) ;
    float3 centre_0 = (float4(light_0->position_0) ).xyz - world_position_3;
    float3 _S63 = centre_0 - across_0;

#line 2845
    (*corners_0)[int(0)] = _S63 - down_0;
    float3 _S64 = centre_0 + across_0;

#line 2846
    (*corners_0)[int(1)] = _S64 - down_0;
    (*corners_0)[int(2)] = _S64 + down_0;
    (*corners_0)[int(3)] = _S63 + down_0;
    return;
}


#line 2598
matrix<float,int(3),int(3)>  ltc_shading_frame_0(float3 normal_7, float3 to_eye_0, float n_dot_v_1)
{
    float3 across_1 = to_eye_0 - normal_7 * float3(n_dot_v_1) ;
    float span_0 = length(across_1);

#line 2601
    float3 seed_0;
    if((abs(normal_7.z)) < 0.89999997615814209f)
    {

#line 2602
        seed_0 = float3(0.0f, 0.0f, 1.0f);

#line 2602
    }
    else
    {

#line 2602
        seed_0 = float3(1.0f, 0.0f, 0.0f);

#line 2602
    }

#line 2602
    float3 tangent_5;
    if(span_0 > 0.00009999999747379f)
    {

#line 2603
        tangent_5 = across_1 / float3(span_0) ;

#line 2603
    }
    else
    {

#line 2603
        tangent_5 = normalize(cross(seed_0, normal_7));

#line 2603
    }

    return matrix<float,int(3),int(3)> (tangent_5, cross(normal_7, tangent_5), normal_7);
}


#line 2579
struct LtcPolygon_0
{
    array<float3, int(5)> corner_0;
    int count_0;
};


#line 2669
LtcPolygon_0 ltc_clip_0(const LtcPolygon_0 thread* polygon_0)
{

#line 2669
    float3 _S65 = polygon_0->corner_0[int(0)];

#line 2669
    float3 _S66 = polygon_0->corner_0[int(1)];

#line 2669
    float3 _S67 = polygon_0->corner_0[int(2)];

#line 2669
    float3 _S68 = polygon_0->corner_0[int(3)];

#line 2675
    float3 _S69 = float3(0.0f, 0.0f, 0.0f);


    float _S70 = polygon_0->corner_0[int(0)].z;

#line 2678
    int count_1;

#line 2678
    if(_S70 > 0.0f)
    {

#line 2678
        count_1 = int(1);

#line 2678
    }
    else
    {

#line 2678
        count_1 = int(0);

#line 2678
    }
    float _S71 = _S66.z;

#line 2679
    int _S72;

#line 2679
    if(_S71 > 0.0f)
    {

#line 2679
        _S72 = int(2);

#line 2679
    }
    else
    {

#line 2679
        _S72 = int(0);

#line 2679
    }

#line 2679
    int config_0 = count_1 + _S72;
    float _S73 = _S67.z;

#line 2680
    if(_S73 > 0.0f)
    {

#line 2680
        count_1 = int(4);

#line 2680
    }
    else
    {

#line 2680
        count_1 = int(0);

#line 2680
    }

#line 2680
    int config_1 = config_0 + count_1;
    float _S74 = _S68.z;

#line 2681
    if(_S74 > 0.0f)
    {

#line 2681
        count_1 = int(8);

#line 2681
    }
    else
    {

#line 2681
        count_1 = int(0);

#line 2681
    }

#line 2681
    int config_2 = config_1 + count_1;

#line 2681
    float3 l0_0;

#line 2681
    float3 l1_0;

#line 2681
    float3 l2_0;

#line 2681
    float3 l3_0;

#line 2681
    float3 l4_0;


    if(config_2 == int(1))
    {

#line 2684
        float3 _S75 = float3(_S70) ;


        float3 _S76 = float3(- _S71)  * _S65 + _S75 * _S66;
        float3 _S77 = float3(- _S74)  * _S65 + _S75 * _S68;

#line 2688
        count_1 = int(3);

#line 2688
        l0_0 = _S65;

#line 2688
        l1_0 = _S76;

#line 2688
        l2_0 = _S77;

#line 2688
        l3_0 = _S68;

#line 2688
        l4_0 = _S69;

#line 2684
    }
    else
    {



        if(config_2 == int(2))
        {

#line 2690
            float3 _S78 = float3(_S71) ;


            float3 _S79 = float3(- _S70)  * _S66 + _S78 * _S65;
            float3 _S80 = float3(- _S73)  * _S66 + _S78 * _S67;

#line 2694
            count_1 = int(3);

#line 2694
            l0_0 = _S79;

#line 2694
            l1_0 = _S66;

#line 2694
            l2_0 = _S80;

#line 2694
            l3_0 = _S68;

#line 2694
            l4_0 = _S69;

#line 2690
        }
        else
        {



            if(config_2 == int(3))
            {

                float3 _S81 = float3(- _S73)  * _S66 + float3(_S71)  * _S67;
                float3 _S82 = float3(- _S74)  * _S65 + float3(_S70)  * _S68;

#line 2700
                count_1 = int(4);

#line 2700
                l0_0 = _S65;

#line 2700
                l1_0 = _S66;

#line 2700
                l2_0 = _S81;

#line 2700
                l3_0 = _S82;

#line 2700
                l4_0 = _S69;

#line 2696
            }
            else
            {



                if(config_2 == int(4))
                {

#line 2702
                    float3 _S83 = float3(_S73) ;


                    float3 _S84 = float3(- _S74)  * _S67 + _S83 * _S68;
                    float3 _S85 = float3(- _S71)  * _S67 + _S83 * _S66;

#line 2706
                    count_1 = int(3);

#line 2706
                    l0_0 = _S84;

#line 2706
                    l1_0 = _S85;

#line 2706
                    l2_0 = _S67;

#line 2706
                    l3_0 = _S68;

#line 2706
                    l4_0 = _S69;

#line 2702
                }
                else
                {



                    if(config_2 == int(6))
                    {

                        float3 _S86 = float3(- _S70)  * _S66 + float3(_S71)  * _S65;
                        float3 _S87 = float3(- _S74)  * _S67 + float3(_S73)  * _S68;

#line 2712
                        count_1 = int(4);

#line 2712
                        l0_0 = _S86;

#line 2712
                        l1_0 = _S66;

#line 2712
                        l2_0 = _S67;

#line 2712
                        l3_0 = _S87;

#line 2712
                        l4_0 = _S69;

#line 2708
                    }
                    else
                    {



                        if(config_2 == int(7))
                        {

#line 2714
                            float3 _S88 = float3(- _S74) ;


                            float3 _S89 = _S88 * _S65 + float3(_S70)  * _S68;
                            float3 _S90 = _S88 * _S67 + float3(_S73)  * _S68;

#line 2718
                            count_1 = int(5);

#line 2718
                            l0_0 = _S65;

#line 2718
                            l1_0 = _S66;

#line 2718
                            l2_0 = _S67;

#line 2718
                            l3_0 = _S90;

#line 2718
                            l4_0 = _S89;

#line 2714
                        }
                        else
                        {



                            if(config_2 == int(8))
                            {

#line 2720
                                float3 _S91 = float3(_S74) ;


                                float3 _S92 = float3(- _S70)  * _S68 + _S91 * _S65;
                                float3 _S93 = float3(- _S73)  * _S68 + _S91 * _S67;

#line 2724
                                count_1 = int(3);

#line 2724
                                l0_0 = _S92;

#line 2724
                                l1_0 = _S93;

#line 2724
                                l2_0 = _S68;

#line 2724
                                l3_0 = _S68;

#line 2724
                                l4_0 = _S69;

#line 2720
                            }
                            else
                            {

#line 2727
                                if(config_2 == int(9))
                                {

                                    float3 _S94 = float3(- _S71)  * _S65 + float3(_S70)  * _S66;
                                    float3 _S95 = float3(- _S73)  * _S68 + float3(_S74)  * _S67;

#line 2731
                                    count_1 = int(4);

#line 2731
                                    l0_0 = _S65;

#line 2731
                                    l1_0 = _S94;

#line 2731
                                    l2_0 = _S95;

#line 2731
                                    l3_0 = _S68;

#line 2731
                                    l4_0 = _S69;

#line 2727
                                }
                                else
                                {



                                    if(config_2 == int(11))
                                    {


                                        float3 _S96 = float3(- _S74)  * _S67 + float3(_S73)  * _S68;
                                        float3 _S97 = float3(- _S73)  * _S66 + float3(_S71)  * _S67;

#line 2738
                                        count_1 = int(5);

#line 2738
                                        l0_0 = _S65;

#line 2738
                                        l1_0 = _S66;

#line 2738
                                        l2_0 = _S97;

#line 2738
                                        l3_0 = _S96;

#line 2738
                                        l4_0 = _S68;

#line 2733
                                    }
                                    else
                                    {

#line 2740
                                        if(config_2 == int(12))
                                        {

                                            float3 _S98 = float3(- _S71)  * _S67 + float3(_S73)  * _S66;
                                            float3 _S99 = float3(- _S70)  * _S68 + float3(_S74)  * _S65;

#line 2744
                                            count_1 = int(4);

#line 2744
                                            l0_0 = _S99;

#line 2744
                                            l1_0 = _S98;

#line 2744
                                            l2_0 = _S67;

#line 2744
                                            l3_0 = _S68;

#line 2744
                                            l4_0 = _S69;

#line 2740
                                        }
                                        else
                                        {



                                            if(config_2 == int(13))
                                            {



                                                float3 _S100 = float3(- _S73)  * _S66 + float3(_S71)  * _S67;
                                                float3 _S101 = float3(- _S71)  * _S65 + float3(_S70)  * _S66;

#line 2752
                                                count_1 = int(5);

#line 2752
                                                l0_0 = _S65;

#line 2752
                                                l1_0 = _S101;

#line 2752
                                                l2_0 = _S100;

#line 2752
                                                l3_0 = _S67;

#line 2752
                                                l4_0 = _S68;

#line 2746
                                            }
                                            else
                                            {

#line 2754
                                                if(config_2 == int(14))
                                                {

#line 2754
                                                    float3 _S102 = float3(- _S70) ;


                                                    float3 _S103 = _S102 * _S68 + float3(_S74)  * _S65;
                                                    float3 _S104 = _S102 * _S66 + float3(_S71)  * _S65;

#line 2758
                                                    count_1 = int(5);

#line 2758
                                                    l0_0 = _S104;

#line 2758
                                                    l1_0 = _S103;

#line 2754
                                                }
                                                else
                                                {



                                                    if(config_2 == int(15))
                                                    {

#line 2760
                                                        count_1 = int(4);

#line 2760
                                                    }
                                                    else
                                                    {

#line 2760
                                                        count_1 = int(0);

#line 2760
                                                    }

#line 2760
                                                    l0_0 = _S65;

#line 2760
                                                    l1_0 = _S69;

#line 2754
                                                }

#line 2675
                                                float3 _S105 = l1_0;

#line 2675
                                                l1_0 = _S66;

#line 2675
                                                l2_0 = _S67;

#line 2675
                                                l3_0 = _S68;

#line 2675
                                                l4_0 = _S105;

#line 2746
                                            }

#line 2740
                                        }

#line 2733
                                    }

#line 2727
                                }

#line 2720
                            }

#line 2714
                        }

#line 2708
                    }

#line 2702
                }

#line 2696
            }

#line 2690
        }

#line 2684
    }

#line 2768
    if(count_1 <= int(3))
    {

#line 2768
        l3_0 = l0_0;

#line 2768
        l4_0 = l0_0;

#line 2768
    }
    else
    {


        if(count_1 == int(4))
        {

#line 2773
            l4_0 = l0_0;

#line 2773
        }

#line 2768
    }

#line 2778
    thread LtcPolygon_0 clipped_0;
    (&clipped_0)->corner_0[int(0)] = l0_0;
    (&clipped_0)->corner_0[int(1)] = l1_0;
    (&clipped_0)->corner_0[int(2)] = l2_0;
    (&clipped_0)->corner_0[int(3)] = l3_0;
    (&clipped_0)->corner_0[int(4)] = l4_0;
    (&clipped_0)->count_0 = count_1;
    return clipped_0;
}


#line 2641
float ltc_edge_0(float3 first_0, float3 second_0)
{
    float cosine_0 = clamp(dot(first_0, second_0), -1.0f, 1.0f);
    float y_0 = abs(cosine_0);


    float fit_0 = (0.85439848899841309f + (0.49651551246643066f + 0.01452060043811798f * y_0) * y_0) / (3.41759395599365234f + (4.16167259216308594f + y_0) * y_0);

#line 2647
    float weight_1;

#line 2652
    if(cosine_0 > 0.0f)
    {

#line 2652
        weight_1 = fit_0;

#line 2652
    }
    else
    {

#line 2652
        weight_1 = 0.5f / sqrt(max(1.0f - cosine_0 * cosine_0, 1.00000001168609742e-07f)) - fit_0;

#line 2652
    }
    return (first_0.x * second_0.y - first_0.y * second_0.x) * weight_1;
}


#line 2798
float ltc_irradiance_0(matrix<float,int(3),int(3)>  transform_1, const array<float3, int(4)> thread* corners_1)
{
    thread LtcPolygon_0 polygon_1;

#line 2800
    int corner_1 = int(0);
    for(;;)
    {

#line 2801
        if(corner_1 < int(4))
        {
        }
        else
        {

#line 2801
            break;
        }
        (&polygon_1)->corner_0[corner_1] = ((((*corners_1)[corner_1]) * (transform_1)));

#line 2801
        corner_1 = corner_1 + int(1);

#line 2801
    }



    (&polygon_1)->corner_0[int(4)] = float3(0.0f, 0.0f, 0.0f);
    (&polygon_1)->count_0 = int(4);

#line 2806
    thread LtcPolygon_0 _S106 = polygon_1;

#line 2806
    LtcPolygon_0 _S107 = ltc_clip_0(&_S106);
    polygon_1 = _S107;
    if(((&polygon_1)->count_0) == int(0))
    {
        return 0.0f;
    }

#line 2810
    int at_2 = int(0);

    for(;;)
    {

#line 2812
        if(at_2 < int(5))
        {
        }
        else
        {

#line 2812
            break;
        }
        (&polygon_1)->corner_0[at_2] = normalize((&polygon_1)->corner_0[at_2]);

#line 2812
        at_2 = at_2 + int(1);

#line 2812
    }

#line 2819
    float sum_0 = ltc_edge_0((&polygon_1)->corner_0[int(0)], (&polygon_1)->corner_0[int(1)]) + ltc_edge_0((&polygon_1)->corner_0[int(1)], (&polygon_1)->corner_0[int(2)]) + ltc_edge_0((&polygon_1)->corner_0[int(2)], (&polygon_1)->corner_0[int(3)]);

#line 2819
    float sum_1;
    if(((&polygon_1)->count_0) >= int(4))
    {

#line 2820
        sum_1 = sum_0 + ltc_edge_0((&polygon_1)->corner_0[int(3)], (&polygon_1)->corner_0[int(4)]);

#line 2820
    }
    else
    {

#line 2820
        sum_1 = sum_0;

#line 2820
    }



    if(((&polygon_1)->count_0) == int(5))
    {

#line 2824
        sum_1 = sum_1 + ltc_edge_0((&polygon_1)->corner_0[int(4)], (&polygon_1)->corner_0[int(0)]);

#line 2824
    }

#line 2831
    return max(sum_1, 0.0f) * 3.14159274101257324f;
}


#line 2527
float4 ltc_at_0(const TableTap_0 thread* tap_2, KernelContext_0 thread* kernelContext_12)
{
    int _S108 = tap_2->lo_0.x;

#line 2529
    int _S109 = tap_2->lo_0.y;

#line 2529
    int3 _S110 = int3(_S108, _S109, int(0));
    int _S111 = tap_2->hi_0.x;

#line 2530
    int3 _S112 = int3(_S111, _S109, int(0));
    float4 _S113 = float4(tap_2->weight_0.x) ;
    int _S114 = tap_2->hi_0.y;

#line 2532
    int3 _S115 = int3(_S108, _S114, int(0));
    int3 _S116 = int3(_S111, _S114, int(0));

    return mix(mix(((kernelContext_12->ltc_matrix_0).read(vec<uint,2>(((_S110)).xy), uint(((_S110)).z))), ((kernelContext_12->ltc_matrix_0).read(vec<uint,2>(((_S112)).xy), uint(((_S112)).z))), _S113), mix(((kernelContext_12->ltc_matrix_0).read(vec<uint,2>(((_S115)).xy), uint(((_S115)).z))), ((kernelContext_12->ltc_matrix_0).read(vec<uint,2>(((_S116)).xy), uint(((_S116)).z))), _S113), float4(tap_2->weight_0.y) );
}


#line 2614
matrix<float,int(3),int(3)>  ltc_transform_0(float4 entry_0)
{
    return matrix<float,int(3),int(3)> (entry_0.x, 0.0f, entry_0.y, 0.0f, 1.0f, 0.0f, entry_0.z, 0.0f, entry_0.w);
}


#line 2351
float3 ggx_lobe_0(float alpha2_0, float3 f0_0, float n_dot_l_0, float n_dot_v_2, float n_dot_h_0, float v_dot_h_0)
{

#line 2358
    float shape_0 = n_dot_h_0 * n_dot_h_0 * (alpha2_0 - 1.0f) + 1.0f;

#line 2365
    float _S117 = 1.0f - alpha2_0;

#line 2370
    float grazing_0 = 1.0f - v_dot_h_0;
    float grazing2_0 = grazing_0 * grazing_0;


    return float3((alpha2_0 / max(shape_0 * shape_0, 9.99999993922529029e-09f) * (0.5f / max(n_dot_l_0 * sqrt(n_dot_v_2 * n_dot_v_2 * _S117 + alpha2_0) + n_dot_v_2 * sqrt(n_dot_l_0 * n_dot_l_0 * _S117 + alpha2_0), 9.99999997475242708e-07f))))  * (f0_0 + (float3(1.0f, 1.0f, 1.0f) - f0_0) * float3((grazing2_0 * grazing2_0 * grazing_0)) );
}


#line 3443
float4 atlas_rect_0(uint tile_0, KernelContext_0 thread* kernelContext_13)
{
    return kernelContext_13->frame_0->shadow_atlas_rect_0[tile_0];
}


#line 3443
float4 atlas_rect_1(uint tile_1, KernelContext_0 thread* kernelContext_14)
{
    return kernelContext_14->frame_0->shadow_atlas_rect_0[tile_1];
}


#line 3503
bool atlas_rect_is_empty_0(float4 rect_0)
{
    return !((rect_0.x) > 0.0f);
}


#line 3475
float tile_texels_0(float4 rect_1, KernelContext_0 thread* kernelContext_15)
{
    return rect_1.x / kernelContext_15->frame_0->shadow_params_0.x;
}


#line 3072
float shadow_normal_offset_0(float3 geometric_normal_0, float3 to_light_1)
{
    float cosine_1 = saturate(dot(geometric_normal_0, to_light_1));
    return sqrt(saturate(1.0f - cosine_1 * cosine_1));
}


#line 3430
uint shadow_filter_mode_0(float2 pixel_1, KernelContext_0 thread* kernelContext_16)
{

#line 3430
    uint _S118;

    if(uint(pixel_1.x) < (kernelContext_16->frame_0->shadow_filter_0.z))
    {

#line 3432
        _S118 = kernelContext_16->frame_0->shadow_filter_0.x;

#line 3432
    }
    else
    {

#line 3432
        _S118 = kernelContext_16->frame_0->shadow_filter_0.y;

#line 3432
    }

#line 3432
    return _S118;
}


#line 3455
float2 atlas_step_0(float4 rect_2, KernelContext_0 thread* kernelContext_17)
{
    return kernelContext_17->frame_0->shadow_params_0.xy / rect_2.xy;
}


#line 3455
float2 atlas_step_1(float4 rect_3, KernelContext_0 thread* kernelContext_18)
{
    return kernelContext_18->frame_0->shadow_params_0.xy / rect_3.xy;
}


#line 349
float2 atlas_uv_0(float4 rect_4, float2 tile_uv_0)
{
    return rect_4.zw + tile_uv_0 * rect_4.xy;
}


#line 3525
float tile_tap_0(float4 rect_5, float2 texel_step_0, float2 tile_uv_1, float2 spoke_0, float2 rotation_0, float reference_0, KernelContext_0 thread* kernelContext_19)
{

    float2 tile_min_0 = float2(0.5f, 0.5f) * texel_step_0;

    float _S119 = spoke_0.x;

#line 3530
    float _S120 = rotation_0.x;

#line 3530
    float _S121 = spoke_0.y;

#line 3530
    float _S122 = rotation_0.y;


    float _S123 = ((kernelContext_19->shadow_atlas_0).sample_compare((kernelContext_19->shadow_sampler_0), (atlas_uv_0(rect_5, clamp(tile_uv_1 + float2(_S119 * _S120 - _S121 * _S122, _S119 * _S122 + _S121 * _S120) * texel_step_0, tile_min_0, float2(1.0f)  - tile_min_0))), (reference_0), level((0.0f))));

#line 3533
    return _S123;
}


#line 3613
float tile_box_pcf_0(uint tile_2, float2 tile_uv_2, float reference_1, KernelContext_0 thread* kernelContext_20)
{

#line 3613
    float4 _S124 = atlas_rect_1(tile_2, kernelContext_20);


    if(atlas_rect_is_empty_0(_S124))
    {
        return 1.0f;
    }

#line 3618
    float2 _S125 = atlas_step_1(_S124, kernelContext_20);

#line 3618
    int y_1 = int(-1);

#line 3618
    float visibility_0 = 0.0f;

#line 3623
    for(;;)
    {

#line 3623
        if(y_1 <= int(1))
        {
        }
        else
        {

#line 3623
            break;
        }

#line 3623
        int x_0 = int(-1);

        for(;;)
        {

#line 3625
            if(x_0 <= int(1))
            {
            }
            else
            {

#line 3625
                break;
            }

#line 3625
            float _S126 = tile_tap_0(_S124, _S125, tile_uv_2, float2(float(x_0), float(y_1)), float2(1.0f, 0.0f), reference_1, kernelContext_20);

            float visibility_1 = visibility_0 + _S126;

#line 3625
            x_0 = x_0 + int(1);

#line 3625
            visibility_0 = visibility_1;

#line 3625
        }

#line 3623
        y_1 = y_1 + int(1);

#line 3623
    }

#line 3631
    return visibility_0 / 9.0f;
}


#line 3388
float2 shadow_rotation_0(float2 pixel_2)
{
    uint2 cell_0 = uint2(pixel_2) & (uint2(3U) );
    return SHADOW_ROTATIONS_0[SHADOW_DITHER_0[cell_0.y * 4U + cell_0.x]];
}


#line 3555
float tile_pcf_0(uint tile_3, float2 tile_uv_3, float reference_2, float2 pixel_3, float radius_2, KernelContext_0 thread* kernelContext_21)
{
    float2 _S127 = shadow_rotation_0(pixel_3);

#line 3557
    float4 _S128 = atlas_rect_1(tile_3, kernelContext_21);

    if(atlas_rect_is_empty_0(_S128))
    {
        return 1.0f;
    }

#line 3561
    float2 _S129 = atlas_step_1(_S128, kernelContext_21);

#line 3561
    uint spot_0 = 0U;

#line 3561
    float probe_0 = 0.0f;

#line 3566
    for(;;)
    {

#line 3566
        if(spot_0 < 5U)
        {
        }
        else
        {

#line 3566
            break;
        }

#line 3566
        float _S130 = tile_tap_0(_S128, _S129, tile_uv_3, SHADOW_DISC_0[SHADOW_PROBE_INDEX_0[spot_0]] * float2(radius_2) , _S127, reference_2, kernelContext_21);

        float probe_1 = probe_0 + _S130;

#line 3566
        spot_0 = spot_0 + 1U;

#line 3566
        probe_0 = probe_1;

#line 3566
    }

#line 3575
    if(probe_0 <= 0.0f)
    {
        return 0.0f;
    }
    if(probe_0 >= 5.0f)
    {
        return 1.0f;
    }

#line 3581
    uint index_2 = 0U;

#line 3581
    float visibility_2 = 0.0f;



    for(;;)
    {

#line 3585
        if(index_2 < 32U)
        {
        }
        else
        {

#line 3585
            break;
        }

#line 3585
        float _S131 = tile_tap_0(_S128, _S129, tile_uv_3, SHADOW_DISC_0[index_2] * float2(radius_2) , _S127, reference_2, kernelContext_21);

        float visibility_3 = visibility_2 + _S131;

#line 3585
        index_2 = index_2 + 1U;

#line 3585
        visibility_2 = visibility_3;

#line 3585
    }

#line 3590
    return visibility_2 / 32.0f;
}


#line 3666
float sun_penumbra_texels_0(uint cascade_0, float2 tile_uv_4, float reference_3, float2 rotation_1, KernelContext_0 thread* kernelContext_22)
{
    float2 texel_1 = kernelContext_22->frame_0->shadow_params_0.xy;

#line 3668
    float4 _S132 = atlas_rect_0(cascade_0, kernelContext_22);

#line 3668
    float2 _S133 = atlas_step_0(_S132, kernelContext_22);


    float2 _S134 = float2(0.5f, 0.5f) * _S133;


    float2 _S135 = float2(1.0f, 1.0f);

#line 3674
    float2 _S136 = _S135 / texel_1;

#line 3674
    uint index_3 = 0U;

#line 3674
    float sum_2 = 0.0f;

#line 3674
    float found_0 = 0.0f;



    for(;;)
    {

#line 3678
        if(index_3 < 16U)
        {
        }
        else
        {

#line 3678
            break;
        }
        float2 spoke_1 = SHADOW_SEARCH_DISC_0[index_3] * float2(8.0f) ;
        float _S137 = spoke_1.x;

#line 3681
        float _S138 = rotation_1.x;

#line 3681
        float _S139 = spoke_1.y;

#line 3681
        float _S140 = rotation_1.y;

#line 3689
        int3 _S141 = int3(int2(min(atlas_uv_0(_S132, clamp(tile_uv_4 + float2(_S137 * _S138 - _S139 * _S140, _S137 * _S140 + _S139 * _S138) * _S133, _S134, float2(1.0f)  - _S134)) * _S136, _S136 - _S135)), int(0));

#line 3689
        float depth_1 = ((kernelContext_22->shadow_atlas_0).read(vec<uint,2>(((_S141)).xy), uint(((_S141)).z)));
        if(depth_1 > reference_3)
        {

            float found_1 = found_0 + 1.0f;

#line 3693
            sum_2 = sum_2 + depth_1;

#line 3693
            found_0 = found_1;

#line 3690
        }

#line 3678
        index_3 = index_3 + 1U;

#line 3678
    }

#line 3697
    if(found_0 <= 0.0f)
    {
        return 2.0f;
    }

#line 3708
    float _S142 = 2.0f * kernelContext_22->frame_0->cascade_far_0[cascade_0];

#line 3708
    float separation_0 = (sum_2 / found_0 - reference_3) * (_S142 + 40.0f);

#line 3708
    float _S143 = tile_texels_0(_S132, kernelContext_22);

    return clamp(separation_0 * 0.01999999955296516f / (_S142 / _S143), 2.0f, 8.0f);
}


#line 3762
float cascade_visibility_0(uint cascade_1, float3 world_position_4, float3 to_light_2, float3 geometric_normal_1, float2 pixel_4, KernelContext_0 thread* kernelContext_23)
{

#line 3763
    float4 _S144 = atlas_rect_0(cascade_1, kernelContext_23);

#line 3797
    if(atlas_rect_is_empty_0(_S144))
    {


        return 1.0f;
    }
    float _S145 = 2.0f * kernelContext_23->frame_0->cascade_far_0[cascade_1];

#line 3803
    float _S146 = tile_texels_0(_S144, kernelContext_23);

#line 3803
    float texel_world_0 = _S145 / _S146;

#line 3810
    float4 clip_0 = (((float4(world_position_4 + geometric_normal_1 * float3((texel_world_0 * kernelContext_23->frame_0->shadow_params_0.w * shadow_normal_offset_0(geometric_normal_1, to_light_2)))  + to_light_2 * float3((texel_world_0 * kernelContext_23->frame_0->shadow_params_0.z)) , 1.0f)) * (matrix<float,int(4),int(4)> ((&kernelContext_23->frame_0->shadow_view_proj_0)->data_2[cascade_1].data_1[int(0)][int(0)], (&kernelContext_23->frame_0->shadow_view_proj_0)->data_2[cascade_1].data_1[int(1)][int(0)], (&kernelContext_23->frame_0->shadow_view_proj_0)->data_2[cascade_1].data_1[int(2)][int(0)], (&kernelContext_23->frame_0->shadow_view_proj_0)->data_2[cascade_1].data_1[int(3)][int(0)], (&kernelContext_23->frame_0->shadow_view_proj_0)->data_2[cascade_1].data_1[int(0)][int(1)], (&kernelContext_23->frame_0->shadow_view_proj_0)->data_2[cascade_1].data_1[int(1)][int(1)], (&kernelContext_23->frame_0->shadow_view_proj_0)->data_2[cascade_1].data_1[int(2)][int(1)], (&kernelContext_23->frame_0->shadow_view_proj_0)->data_2[cascade_1].data_1[int(3)][int(1)], (&kernelContext_23->frame_0->shadow_view_proj_0)->data_2[cascade_1].data_1[int(0)][int(2)], (&kernelContext_23->frame_0->shadow_view_proj_0)->data_2[cascade_1].data_1[int(1)][int(2)], (&kernelContext_23->frame_0->shadow_view_proj_0)->data_2[cascade_1].data_1[int(2)][int(2)], (&kernelContext_23->frame_0->shadow_view_proj_0)->data_2[cascade_1].data_1[int(3)][int(2)], (&kernelContext_23->frame_0->shadow_view_proj_0)->data_2[cascade_1].data_1[int(0)][int(3)], (&kernelContext_23->frame_0->shadow_view_proj_0)->data_2[cascade_1].data_1[int(1)][int(3)], (&kernelContext_23->frame_0->shadow_view_proj_0)->data_2[cascade_1].data_1[int(2)][int(3)], (&kernelContext_23->frame_0->shadow_view_proj_0)->data_2[cascade_1].data_1[int(3)][int(3)]))));



    float3 ndc_0 = clip_0.xyz / float3(clip_0.w) ;

#line 3814
    bool _S147;
    if(any((abs(ndc_0.xy)) > (float2(1.0f) )))
    {

#line 3815
        _S147 = true;

#line 3815
    }
    else
    {

#line 3815
        _S147 = (ndc_0.z) <= 0.0f;

#line 3815
    }

#line 3815
    if(_S147)
    {



        return 1.0f;
    }



    float2 tile_uv_5 = float2(ndc_0.x * 0.5f + 0.5f, 0.5f - ndc_0.y * 0.5f);

#line 3825
    uint _S148 = shadow_filter_mode_0(pixel_4, kernelContext_23);

#line 3842
    if(_S148 == 2U)
    {

#line 3842
        float _S149 = tile_box_pcf_0(cascade_1, tile_uv_5, ndc_0.z, kernelContext_23);

        return _S149;
    }
    if(_S148 == 1U)
    {

#line 3846
        float _S150 = tile_pcf_0(cascade_1, tile_uv_5, ndc_0.z, pixel_4, 2.0f, kernelContext_23);



        return _S150;
    }

    float _S151 = ndc_0.z;

#line 3853
    float _S152 = sun_penumbra_texels_0(cascade_1, tile_uv_5, _S151, shadow_rotation_0(pixel_4), kernelContext_23);

#line 3853
    float _S153 = tile_pcf_0(cascade_1, tile_uv_5, _S151, pixel_4, _S152, kernelContext_23);
    return _S153;
}


#line 3933
float sun_visibility_0(float3 world_position_5, float3 to_light_3, float n_dot_l_1, float3 geometric_normal_2, float2 pixel_5, uint thread* selected_0, float thread* fade_0, KernelContext_0 thread* kernelContext_24)
{
    uint cascade_2;

#line 3935
    bool covered_0;

#line 3944
    *selected_0 = 2U;
    *fade_0 = 0.0f;
    if(n_dot_l_1 <= 0.0f)
    {
        return 1.0f;
    }

#line 3956
    float eye_distance_0 = length(world_position_5 - kernelContext_24->frame_0->camera_position_0.xyz);

#line 3956
    uint index_4 = 0U;

#line 3964
    for(;;)
    {

#line 3964
        if(index_4 < 2U)
        {
        }
        else
        {

#line 3964
            covered_0 = false;

#line 3964
            cascade_2 = 1U;

#line 3964
            break;
        }
        if(eye_distance_0 < kernelContext_24->frame_0->cascade_far_0[index_4])
        {

#line 3966
            covered_0 = true;

#line 3966
            cascade_2 = index_4;



            break;
        }

#line 3964
        index_4 = index_4 + 1U;

#line 3964
    }

#line 3973
    if(covered_0)
    {
        *selected_0 = cascade_2;

#line 3973
    }

#line 3973
    float _S154 = cascade_visibility_0(cascade_2, world_position_5, to_light_3, geometric_normal_2, pixel_5, kernelContext_24);

#line 3980
    uint _S155 = cascade_2 + 1U;

#line 3980
    if(_S155 >= 2U)
    {



        return _S154;
    }

#line 3993
    float band_0 = kernelContext_24->frame_0->cascade_far_0[cascade_2] * 0.10000000149011612f;
    float blend_0 = saturate((eye_distance_0 - (kernelContext_24->frame_0->cascade_far_0[cascade_2] - band_0)) / band_0);



    *fade_0 = blend_0;
    if(blend_0 <= 0.0f)
    {
        return _S154;
    }

#line 4001
    float _S156 = cascade_visibility_0(_S155, world_position_5, to_light_3, geometric_normal_2, pixel_5, kernelContext_24);

#line 4012
    return mix(_S154, _S156, blend_0);
}


#line 5200
float contact_at_0(float2 position_4, KernelContext_0 thread* kernelContext_25)
{

#line 5200
    texture2d<float, access::sample> _S157 = kernelContext_25->contact_shadow_0;

    thread uint width_2;
    thread uint height_2;
    (*((&width_2)) = (_S157).get_width(0)),(*((&height_2)) = (_S157).get_height(0));

    int3 _S158 = int3(min(int2(position_4), int2(int(width_2), int(height_2)) - int2(int(1)) ), int(0));

#line 5206
    return ((kernelContext_25->contact_shadow_0).read(vec<uint,2>(((_S158)).xy), uint(((_S158)).z)).x);
}


#line 3905
float3 cascade_tint_0(uint cascade_3, float blend_1)
{
    if(cascade_3 >= 2U)
    {
        return float3(1.0f, 1.0f, 1.0f);
    }
    uint _S159 = cascade_3 + 1U;

#line 3911
    if(_S159 >= 2U)
    {


        return CASCADE_TINTS_0[cascade_3];
    }
    return mix(CASCADE_TINTS_0[cascade_3], CASCADE_TINTS_0[_S159], float3(blend_1) );
}


#line 4223
uint point_face_0(float3 from_light_0)
{
    float3 axis_2 = abs(from_light_0);
    float _S160 = axis_2.x;

#line 4226
    float _S161 = axis_2.y;

#line 4226
    bool _S162;

#line 4226
    if(_S160 >= _S161)
    {

#line 4226
        _S162 = _S160 >= (axis_2.z);

#line 4226
    }
    else
    {

#line 4226
        _S162 = false;

#line 4226
    }

#line 4226
    uint _S163;

#line 4226
    if(_S162)
    {
        if((from_light_0.x) >= 0.0f)
        {

#line 4228
            _S163 = 0U;

#line 4228
        }
        else
        {

#line 4228
            _S163 = 1U;

#line 4228
        }

#line 4228
        return _S163;
    }
    if(_S161 >= (axis_2.z))
    {
        if((from_light_0.y) >= 0.0f)
        {

#line 4232
            _S163 = 2U;

#line 4232
        }
        else
        {

#line 4232
            _S163 = 3U;

#line 4232
        }

#line 4232
        return _S163;
    }
    if((from_light_0.z) >= 0.0f)
    {

#line 4234
        _S163 = 4U;

#line 4234
    }
    else
    {

#line 4234
        _S163 = 5U;

#line 4234
    }

#line 4234
    return _S163;
}


#line 336
uint light_tile_0(uint tile_4)
{
    return 2U + tile_4;
}


#line 4119
float punctual_visibility_0(uint tile_5, float3 world_position_6, float3 to_light_4, float n_dot_l_2, float map_world_0, float3 geometric_normal_3, float2 pixel_6, KernelContext_0 thread* kernelContext_26)
{

    uint atlas_0 = light_tile_0(tile_5);

#line 4122
    float4 _S164 = atlas_rect_0(atlas_0, kernelContext_26);

    if(atlas_rect_is_empty_0(_S164))
    {


        return 1.0f;
    }

#line 4128
    float _S165 = tile_texels_0(_S164, kernelContext_26);

    float texel_world_1 = map_world_0 / _S165;

#line 4140
    float4 clip_1 = (((float4(world_position_6 + geometric_normal_3 * float3((texel_world_1 * 4.0f * shadow_normal_offset_0(geometric_normal_3, to_light_4)))  + to_light_4 * float3((texel_world_1 * 2.0f)) , 1.0f)) * (matrix<float,int(4),int(4)> ((&kernelContext_26->frame_0->light_view_proj_0)->data_3[tile_5].data_1[int(0)][int(0)], (&kernelContext_26->frame_0->light_view_proj_0)->data_3[tile_5].data_1[int(1)][int(0)], (&kernelContext_26->frame_0->light_view_proj_0)->data_3[tile_5].data_1[int(2)][int(0)], (&kernelContext_26->frame_0->light_view_proj_0)->data_3[tile_5].data_1[int(3)][int(0)], (&kernelContext_26->frame_0->light_view_proj_0)->data_3[tile_5].data_1[int(0)][int(1)], (&kernelContext_26->frame_0->light_view_proj_0)->data_3[tile_5].data_1[int(1)][int(1)], (&kernelContext_26->frame_0->light_view_proj_0)->data_3[tile_5].data_1[int(2)][int(1)], (&kernelContext_26->frame_0->light_view_proj_0)->data_3[tile_5].data_1[int(3)][int(1)], (&kernelContext_26->frame_0->light_view_proj_0)->data_3[tile_5].data_1[int(0)][int(2)], (&kernelContext_26->frame_0->light_view_proj_0)->data_3[tile_5].data_1[int(1)][int(2)], (&kernelContext_26->frame_0->light_view_proj_0)->data_3[tile_5].data_1[int(2)][int(2)], (&kernelContext_26->frame_0->light_view_proj_0)->data_3[tile_5].data_1[int(3)][int(2)], (&kernelContext_26->frame_0->light_view_proj_0)->data_3[tile_5].data_1[int(0)][int(3)], (&kernelContext_26->frame_0->light_view_proj_0)->data_3[tile_5].data_1[int(1)][int(3)], (&kernelContext_26->frame_0->light_view_proj_0)->data_3[tile_5].data_1[int(2)][int(3)], (&kernelContext_26->frame_0->light_view_proj_0)->data_3[tile_5].data_1[int(3)][int(3)]))));

#line 4147
    float _S166 = clip_1.w;

#line 4147
    if(_S166 <= 0.0f)
    {
        return 1.0f;
    }
    float3 ndc_1 = clip_1.xyz / float3(_S166) ;

#line 4151
    bool _S167;
    if(any((abs(ndc_1.xy)) > (float2(1.0f) )))
    {

#line 4152
        _S167 = true;

#line 4152
    }
    else
    {

#line 4152
        _S167 = (ndc_1.z) <= 0.0f;

#line 4152
    }

#line 4152
    if(_S167)
    {

#line 4152
        _S167 = true;

#line 4152
    }
    else
    {

#line 4152
        _S167 = (ndc_1.z) > 1.0f;

#line 4152
    }

#line 4152
    if(_S167)
    {

#line 4159
        return 1.0f;
    }



    float2 tile_uv_6 = float2(ndc_1.x * 0.5f + 0.5f, 0.5f - ndc_1.y * 0.5f);

#line 4164
    uint _S168 = shadow_filter_mode_0(pixel_6, kernelContext_26);

#line 4173
    if(_S168 == 2U)
    {

#line 4173
        float _S169 = tile_box_pcf_0(atlas_0, tile_uv_6, ndc_1.z, kernelContext_26);

        return _S169;
    }

#line 4175
    float _S170 = tile_pcf_0(atlas_0, tile_uv_6, ndc_1.z, pixel_6, 2.0f, kernelContext_26);

    return _S170;
}


#line 4242
float point_visibility_0(const GpuLight_natural_0 thread* light_1, uint base_1, float3 world_position_7, float3 to_light_5, float n_dot_l_3, float3 geometric_normal_4, float2 pixel_7, KernelContext_0 thread* kernelContext_27)
{

    if(n_dot_l_3 <= 0.0f)
    {
        return 1.0f;
    }

    float3 from_light_1 = world_position_7 - (float4(light_1->position_0) ).xyz;

#line 4250
    float _S171 = punctual_visibility_0(base_1 + point_face_0(from_light_1), world_position_7, to_light_5, n_dot_l_3, 2.0f * max(max(abs(from_light_1.x), abs(from_light_1.y)), abs(from_light_1.z)), geometric_normal_4, pixel_7, kernelContext_27);

#line 4256
    return _S171;
}


#line 4184
float spot_visibility_0(const GpuLight_natural_0 thread* light_2, uint tile_6, float3 world_position_8, float3 to_light_6, float n_dot_l_4, float3 geometric_normal_5, float2 pixel_8, KernelContext_0 thread* kernelContext_28)
{

    if(n_dot_l_4 <= 0.0f)
    {


        return 1.0f;
    }

#line 4191
    float4 _S172 = float4(light_2->direction_0) ;

#line 4198
    float cos_outer_1 = _S172.w;

#line 4198
    float _S173 = punctual_visibility_0(tile_6, world_position_8, to_light_6, n_dot_l_4, 2.0f * (sqrt(saturate(1.0f - cos_outer_1 * cos_outer_1)) / max(cos_outer_1, 0.00009999999747379f)) * max(dot(world_position_8 - (float4(light_2->position_0) ).xyz, normalize(_S172.xyz)), 0.0f), geometric_normal_5, pixel_8, kernelContext_28);

#line 4205
    return _S173;
}


#line 2555
float3 specular_compensation_0(float3 f0_1, float directional_albedo_0)
{


    return float3(1.0f, 1.0f, 1.0f) + f0_1 * float3((1.0f / clamp(directional_albedo_0, 0.00009999999747379f, 1.0f) - 1.0f)) ;
}


#line 5187
float3 bent_normal_at_0(float4 occlusion_0, float3 shading_normal_1)
{
    float3 decoded_0 = occlusion_0.yzw * float3(2.0f)  - float3(1.0f) ;

#line 5189
    float3 _S174;
    if((length(decoded_0)) < 0.5f)
    {

#line 5190
        _S174 = shading_normal_1;

#line 5190
    }
    else
    {

#line 5190
        _S174 = normalize(decoded_0);

#line 5190
    }

#line 5190
    return _S174;
}


#line 4825
float3 sky_irradiance_0(float3 normal_8, KernelContext_0 thread* kernelContext_29)
{
    float4 basis_6 = float4(normal_8, 1.0f);
    return max(float3(dot(kernelContext_29->frame_0->sky_sh_r_0, basis_6), dot(kernelContext_29->frame_0->sky_sh_g_0, basis_6), dot(kernelContext_29->frame_0->sky_sh_b_0, basis_6)), float3(0.0f, 0.0f, 0.0f));
}


#line 4729
float probe_level_reach_0(float3 world_position_9, float3 origin_0, float3 inv_spacing_0, float3 last_0)
{

#line 4729
    float reach_0 = 0.0f;

#line 4729
    uint axis_3 = 0U;


    for(;;)
    {

#line 4732
        if(axis_3 < 3U)
        {
        }
        else
        {

#line 4732
            break;
        }

#line 4732
        uint _S175 = axis_3;

#line 4732
        bool _S176;

        if((last_0[axis_3]) == 0.0f)
        {

#line 4734
            _S176 = true;

#line 4734
        }
        else
        {

#line 4734
            _S176 = (inv_spacing_0[axis_3]) == 0.0f;

#line 4734
        }

#line 4734
        if(_S176)
        {

#line 4735
            axis_3 = axis_3 + 1U;

#line 4732
            continue;
        }

#line 4732
        reach_0 = max(reach_0, abs(2.0f * ((world_position_9[axis_3] - origin_0[axis_3]) * inv_spacing_0[axis_3]) / last_0[_S175] - 1.0f));

#line 4732
        axis_3 = axis_3 + 1U;

#line 4732
    }

#line 4739
    return reach_0;
}


#line 4759
float2 probe_level_of_0(float reach_1, uint levels_0)
{

#line 4759
    uint level_0 = 0U;

    for(;;)
    {

#line 4761
        uint _S177 = level_0 + 1U;

#line 4761
        if(_S177 < levels_0)
        {
        }
        else
        {

#line 4761
            break;
        }
        float _S178 = float(level_0);

#line 4763
        float at_3 = reach_1 * exp2(- _S178);
        if(at_3 < 1.0f)
        {

#line 4765
            return float2(_S178, saturate((1.0f - at_3) / 0.25f));
        }

#line 4761
        level_0 = _S177;

#line 4761
    }

#line 4767
    return float2(float(levels_0 - 1U), 1.0f);
}


#line 4516
uint probe_wrap_0(uint cell_1, uint offset_0, uint count_2)
{
    uint at_4 = cell_1 + offset_0;

#line 4518
    uint _S179;
    if(at_4 >= count_2)
    {

#line 4519
        _S179 = at_4 - count_2;

#line 4519
    }
    else
    {

#line 4519
        _S179 = at_4;

#line 4519
    }

#line 4519
    return _S179;
}


#line 4542
uint probe_row_0(uint level_1, uint3 cell_2, KernelContext_0 thread* kernelContext_30)
{
    uint3 counts_0 = kernelContext_30->frame_0->probe_counts_0.xyz;
    uint3 offset_1 = kernelContext_30->frame_0->probe_level_offset_0[level_1].xyz;
    uint _S180 = counts_0.x;
    uint _S181 = counts_0.y;



    return min(kernelContext_30->frame_0->probe_levels_0.y * level_1 + (probe_wrap_0(cell_2.z, offset_1.z, counts_0.z) * _S181 + probe_wrap_0(cell_2.y, offset_1.y, _S181)) * _S180 + probe_wrap_0(cell_2.x, offset_1.x, _S180), max(kernelContext_30->frame_0->probe_counts_0.w, 1U) - 1U);
}


#line 4383
float sign_not_zero_0(float value_0)
{

#line 4383
    float _S182;

    if(value_0 >= 0.0f)
    {

#line 4385
        _S182 = 1.0f;

#line 4385
    }
    else
    {

#line 4385
        _S182 = -1.0f;

#line 4385
    }

#line 4385
    return _S182;
}


#line 4402
float2 oct_encode_0(float3 direction_1)
{
    float _S183 = direction_1.y;
    float2 p_0 = direction_1.xz / float2(max(abs(direction_1.x) + abs(_S183) + abs(direction_1.z), 9.99999968265522539e-21f)) ;

#line 4405
    float2 p_1;
    if(_S183 < 0.0f)
    {
        float _S184 = p_0.y;

#line 4408
        float _S185 = p_0.x;

#line 4408
        p_1 = float2((1.0f - abs(_S184)) * sign_not_zero_0(_S185), (1.0f - abs(_S185)) * sign_not_zero_0(_S184));

#line 4406
    }
    else
    {

#line 4406
        p_1 = p_0;

#line 4406
    }

#line 4411
    return p_1;
}


#line 4431
float2 probe_moments_0(uint index_5, float3 direction_2, KernelContext_0 thread* kernelContext_31)
{

#line 4431
    texture2d_array<float, access::sample> _S186 = kernelContext_31->probe_visibility_0;

    thread uint width_3;
    thread uint height_3;
    thread uint layers_0;
    (*((&width_3)) = (_S186).get_width(0)),(*((&height_3)) = (_S186).get_height(0)),(*((&layers_0)) = (_S186).get_array_size());

#line 4436
    float2 _S187 = float2(0.5f) ;

#line 4436
    float2 _S188 = float2(1.0f) ;


    float2 scaled_1 = (oct_encode_0(direction_2) * _S187 + _S187) * float2(16.0f)  + _S188 - _S187;
    float2 _S189 = float2(float(width_3), float(height_3)) - _S188;

#line 4440
    float2 low_2 = clamp(floor(scaled_1), float2(0.0f, 0.0f), _S189);
    float2 high_2 = min(low_2 + _S188, _S189);
    float2 weight_2 = clamp(scaled_1 - low_2, float2(0.0f) , float2(1.0f) );
    int layer_1 = int(min(index_5, max(layers_0, 1U) - 1U));

    int _S190 = int(low_2.x);

#line 4445
    int _S191 = int(low_2.y);

#line 4445
    int4 _S192 = int4(_S190, _S191, layer_1, int(0));
    int _S193 = int(high_2.x);

#line 4446
    int4 _S194 = int4(_S193, _S191, layer_1, int(0));
    int _S195 = int(high_2.y);

#line 4447
    int4 _S196 = int4(_S190, _S195, layer_1, int(0));
    int4 _S197 = int4(_S193, _S195, layer_1, int(0));
    float2 _S198 = float2(weight_2.x) ;

#line 4449
    return mix(mix(((kernelContext_31->probe_visibility_0).read(vec<uint,2>(((_S192)).xy), uint(((_S192)).z), uint(((_S192)).w))).xy, ((kernelContext_31->probe_visibility_0).read(vec<uint,2>(((_S194)).xy), uint(((_S194)).z), uint(((_S194)).w))).xy, _S198), mix(((kernelContext_31->probe_visibility_0).read(vec<uint,2>(((_S196)).xy), uint(((_S196)).z), uint(((_S196)).w))).xy, ((kernelContext_31->probe_visibility_0).read(vec<uint,2>(((_S197)).xy), uint(((_S197)).z), uint(((_S197)).w))).xy, _S198), float2(weight_2.y) );
}


#line 4477
float probe_chebyshev_0(uint index_6, float3 probe_position_0, float3 world_position_10, float3 normal_9, KernelContext_0 thread* kernelContext_32)
{
    float3 to_probe_0 = probe_position_0 - (world_position_10 + normal_9 * float3(0.05000000074505806f) );
    float to_surface_0 = length(to_probe_0);

#line 4480
    float2 _S199 = probe_moments_0(index_6, - to_probe_0, kernelContext_32);

#line 4486
    float _S200 = _S199.x;

#line 4486
    float _S201 = max(_S199.y - _S200 * _S200, 0.0f);
    float behind_0 = to_surface_0 - _S200;
    float bound_0 = _S201 / (_S201 + behind_0 * behind_0);

#line 4488
    float _S202;
    if(to_surface_0 <= _S200)
    {

#line 4489
        _S202 = 1.0f;

#line 4489
    }
    else
    {

#line 4489
        _S202 = bound_0 * bound_0 * bound_0;

#line 4489
    }

#line 4489
    return _S202;
}


#line 4499
float probe_weight_0(uint index_7, float3 probe_position_1, float3 world_position_11, float3 normal_10, KernelContext_0 thread* kernelContext_33)
{

#line 4499
    float _S203 = probe_chebyshev_0(index_7, probe_position_1, world_position_11, normal_10, kernelContext_33);

    return max(_S203, 0.00009999999747379f);
}


#line 1220
struct GpuProbe_0
{
    float4 sh_r_0;
    float4 sh_g_0;
    float4 sh_b_0;
};


#line 4561
struct WeightedProbe_0
{
    GpuProbe_0 sh_0;
    float weight_3;
};


#line 4588
WeightedProbe_0 probe_corner_0(uint level_2, uint3 cell_3, float3 origin_1, float3 spacing_0, float3 world_position_12, float3 normal_11, KernelContext_0 thread* kernelContext_34)
{

#line 4589
    uint _S204 = probe_row_0(level_2, cell_3, kernelContext_34);


    GpuProbe_natural_0 stored_0 = kernelContext_34->probes_0[_S204];

#line 4592
    float _S205 = probe_weight_0(_S204, origin_1 + float3(cell_3) * spacing_0, world_position_12, normal_11, kernelContext_34);



    thread WeightedProbe_0 corner_2;

#line 4596
    float4 _S206 = float4(_S205) ;
    (&(&corner_2)->sh_0)->sh_r_0 = float4(stored_0.sh_r_0)  * _S206;
    (&(&corner_2)->sh_0)->sh_g_0 = float4(stored_0.sh_g_0)  * _S206;
    (&(&corner_2)->sh_0)->sh_b_0 = float4(stored_0.sh_b_0)  * _S206;
    (&corner_2)->weight_3 = _S205;
    return corner_2;
}


#line 4572
WeightedProbe_0 lerp_probe_0(const WeightedProbe_0 thread* a_1, const WeightedProbe_0 thread* b_0, float t_1)
{
    thread WeightedProbe_0 blended_0;
    float4 _S207 = float4(t_1) ;

#line 4575
    (&(&blended_0)->sh_0)->sh_r_0 = mix((&a_1->sh_0)->sh_r_0, (&b_0->sh_0)->sh_r_0, _S207);
    (&(&blended_0)->sh_0)->sh_g_0 = mix((&a_1->sh_0)->sh_g_0, (&b_0->sh_0)->sh_g_0, _S207);
    (&(&blended_0)->sh_0)->sh_b_0 = mix((&a_1->sh_0)->sh_b_0, (&b_0->sh_0)->sh_b_0, _S207);
    (&blended_0)->weight_3 = mix(a_1->weight_3, b_0->weight_3, t_1);
    return blended_0;
}


#line 4660
float3 probe_level_irradiance_0(uint level_3, float3 world_position_13, float3 normal_12, KernelContext_0 thread* kernelContext_35)
{

#line 4660
    float3 _S208 = float3(1.0f) ;

#line 4665
    float3 _S209 = float3(0.0f, 0.0f, 0.0f);

#line 4665
    float3 last_1 = max(float3(kernelContext_35->frame_0->probe_counts_0.xyz) - _S208, _S209);



    float3 origin_2 = kernelContext_35->frame_0->probe_level_origin_0[level_3].xyz;
    float3 inv_0 = kernelContext_35->frame_0->probe_level_inv_spacing_0[level_3].xyz;
    float3 grid_0 = clamp((world_position_13 - origin_2) * inv_0, _S209, last_1);
    float3 base_2 = floor(grid_0);
    float3 f_0 = grid_0 - base_2;

    uint3 _S210 = uint3(base_2);



    uint3 _S211 = uint3(min(base_2 + _S208, last_1));

#line 4685
    float _S212 = inv_0.x;

#line 4685
    float _S213;

#line 4685
    if(_S212 != 0.0f)
    {

#line 4685
        _S213 = 1.0f / _S212;

#line 4685
    }
    else
    {

#line 4685
        _S213 = 0.0f;

#line 4685
    }
    float _S214 = inv_0.y;

#line 4686
    float _S215;

#line 4686
    if(_S214 != 0.0f)
    {

#line 4686
        _S215 = 1.0f / _S214;

#line 4686
    }
    else
    {

#line 4686
        _S215 = 0.0f;

#line 4686
    }
    float _S216 = inv_0.z;

#line 4687
    float _S217;

#line 4687
    if(_S216 != 0.0f)
    {

#line 4687
        _S217 = 1.0f / _S216;

#line 4687
    }
    else
    {

#line 4687
        _S217 = 0.0f;

#line 4687
    }

#line 4685
    float3 spacing_1 = float3(_S213, _S215, _S217);

#line 4694
    uint _S218 = _S210.x;

#line 4694
    uint _S219 = _S210.y;

#line 4694
    uint _S220 = _S210.z;

#line 4694
    WeightedProbe_0 _S221 = probe_corner_0(level_3, uint3(_S218, _S219, _S220), origin_2, spacing_1, world_position_13, normal_12, kernelContext_35);
    uint _S222 = _S211.x;

#line 4695
    WeightedProbe_0 _S223 = probe_corner_0(level_3, uint3(_S222, _S219, _S220), origin_2, spacing_1, world_position_13, normal_12, kernelContext_35);

#line 4695
    float _S224 = f_0.x;

#line 4695
    thread WeightedProbe_0 _S225 = _S221;

#line 4695
    thread WeightedProbe_0 _S226 = _S223;

#line 4695
    WeightedProbe_0 _S227 = lerp_probe_0(&_S225, &_S226, _S224);
    uint _S228 = _S211.y;

#line 4696
    WeightedProbe_0 _S229 = probe_corner_0(level_3, uint3(_S218, _S228, _S220), origin_2, spacing_1, world_position_13, normal_12, kernelContext_35);

#line 4696
    WeightedProbe_0 _S230 = probe_corner_0(level_3, uint3(_S222, _S228, _S220), origin_2, spacing_1, world_position_13, normal_12, kernelContext_35);

#line 4696
    thread WeightedProbe_0 _S231 = _S229;

#line 4696
    thread WeightedProbe_0 _S232 = _S230;

#line 4696
    WeightedProbe_0 _S233 = lerp_probe_0(&_S231, &_S232, _S224);

    uint _S234 = _S211.z;

#line 4698
    WeightedProbe_0 _S235 = probe_corner_0(level_3, uint3(_S218, _S219, _S234), origin_2, spacing_1, world_position_13, normal_12, kernelContext_35);

#line 4698
    WeightedProbe_0 _S236 = probe_corner_0(level_3, uint3(_S222, _S219, _S234), origin_2, spacing_1, world_position_13, normal_12, kernelContext_35);

#line 4698
    thread WeightedProbe_0 _S237 = _S235;

#line 4698
    thread WeightedProbe_0 _S238 = _S236;

#line 4698
    WeightedProbe_0 _S239 = lerp_probe_0(&_S237, &_S238, _S224);

#line 4698
    WeightedProbe_0 _S240 = probe_corner_0(level_3, uint3(_S218, _S228, _S234), origin_2, spacing_1, world_position_13, normal_12, kernelContext_35);

#line 4698
    WeightedProbe_0 _S241 = probe_corner_0(level_3, uint3(_S222, _S228, _S234), origin_2, spacing_1, world_position_13, normal_12, kernelContext_35);

#line 4698
    thread WeightedProbe_0 _S242 = _S240;

#line 4698
    thread WeightedProbe_0 _S243 = _S241;

#line 4698
    WeightedProbe_0 _S244 = lerp_probe_0(&_S242, &_S243, _S224);



    float _S245 = f_0.y;

#line 4702
    thread WeightedProbe_0 _S246 = _S227;

#line 4702
    thread WeightedProbe_0 _S247 = _S233;

#line 4702
    WeightedProbe_0 _S248 = lerp_probe_0(&_S246, &_S247, _S245);

#line 4702
    thread WeightedProbe_0 _S249 = _S239;

#line 4702
    thread WeightedProbe_0 _S250 = _S244;

#line 4702
    WeightedProbe_0 _S251 = lerp_probe_0(&_S249, &_S250, _S245);

    float _S252 = f_0.z;

#line 4704
    thread WeightedProbe_0 _S253 = _S248;

#line 4704
    thread WeightedProbe_0 _S254 = _S251;

#line 4704
    WeightedProbe_0 _S255 = lerp_probe_0(&_S253, &_S254, _S252);

    float4 basis_7 = float4(normal_12, 1.0f);
    return max(float3(dot(_S255.sh_0.sh_r_0, basis_7), dot(_S255.sh_0.sh_g_0, basis_7), dot(_S255.sh_0.sh_b_0, basis_7)) / float3(_S255.weight_3) , _S209);
}


#line 4794
float3 probe_irradiance_0(float3 world_position_14, float3 normal_13, KernelContext_0 thread* kernelContext_36)
{

#line 4802
    float2 pick_0 = probe_level_of_0(probe_level_reach_0(world_position_14, kernelContext_36->frame_0->probe_level_origin_0[int(0)].xyz, kernelContext_36->frame_0->probe_level_inv_spacing_0[int(0)].xyz, max(float3(kernelContext_36->frame_0->probe_counts_0.xyz) - float3(1.0f) , float3(0.0f, 0.0f, 0.0f))), clamp(kernelContext_36->frame_0->probe_levels_0.x, 1U, 4U));
    uint level_4 = uint(pick_0.x);
    float share_0 = pick_0.y;

#line 4804
    float3 _S256 = probe_level_irradiance_0(level_4, world_position_14, normal_13, kernelContext_36);


    if(share_0 >= 1.0f)
    {

#line 4808
        return _S256;
    }

#line 4808
    float3 _S257 = probe_level_irradiance_0(level_4 + 1U, world_position_14, normal_13, kernelContext_36);

    return _S257 * float3((1.0f - share_0))  + _S256 * float3(share_0) ;
}


#line 5256
float3 multi_bounce_occlusion_0(float visibility_4, float3 albedo_0)
{

#line 5256
    float3 _S258 = float3(visibility_4) ;

#line 5262
    return min(float3(1.0f) , max(_S258, ((_S258 * (float3(2.04040002822875977f)  * albedo_0 - float3(0.33239999413490295f) ) + (float3(-4.79510021209716797f)  * albedo_0 + float3(0.64170002937316895f) )) * _S258 + (float3(2.75519990921020508f)  * albedo_0 + float3(0.69029998779296875f) )) * _S258));
}


#line 1115
float3 emissive_of_0(const GpuMaterial_natural_0 thread* material_12)
{
    return float3(material_12->emissive_r_0, material_12->emissive_g_0, material_12->emissive_b_0);
}


#line 2906
float fog_exp_neg_0(float x_1)
{
    float clamped_0 = clamp(x_1, -87.0f, 87.0f);


    float n_0 = floor(clamped_0 * 1.4426950216293335f + 0.5f);


    float _S259 = - (clamped_0 - n_0 * 0.693115234375f - n_0 * 0.00003194618329871f);

#line 2914
    float kernel_0 = 0.0001984127011383f;

#line 2914
    int term_0 = int(6);

    for(;;)
    {

#line 2916
        if(term_0 >= int(0))
        {
        }
        else
        {

#line 2916
            break;
        }
        float _S260 = kernel_0 * _S259 + FOG_KERNEL_0[term_0];

#line 2916
        int term_1 = term_0 - int(1);

#line 2916
        kernel_0 = _S260;

#line 2916
        term_0 = term_1;

#line 2916
    }

#line 2923
    return kernel_0 * (as_type<float>((uint(int(127) - int(n_0)) << 23U)));
}


#line 2933
float fog_one_minus_exp_over_0(float d_0)
{
    if((abs(d_0)) < 0.125f)
    {
        float _S261 = - d_0;

#line 2937
        float series_0 = 0.00833333376795053f;

#line 2937
        int term_2 = int(3);

        for(;;)
        {

#line 2939
            if(term_2 >= int(0))
            {
            }
            else
            {

#line 2939
                break;
            }
            float _S262 = series_0 * _S261 + FOG_RATIO_KERNEL_0[term_2];

#line 2939
            int term_3 = term_2 - int(1);

#line 2939
            series_0 = _S262;

#line 2939
            term_2 = term_3;

#line 2939
        }



        return series_0;
    }
    return (1.0f - fog_exp_neg_0(d_0)) / d_0;
}


#line 2967
float fog_optical_depth_0(float density_0, float falloff_0, float height_a_0, float height_b_0, float distance_2)
{

    if(falloff_0 <= 0.0f)
    {
        return clamp(density_0 * distance_2, 0.0f, 32.0f);
    }

#line 2978
    return clamp(density_0 * distance_2 * fog_exp_neg_0(height_a_0 / falloff_0) * fog_one_minus_exp_over_0((height_b_0 - height_a_0) / falloff_0), 0.0f, 32.0f);
}


#line 2986
float fog_transmittance_0(float optical_depth_0)
{
    return fog_exp_neg_0(max(optical_depth_0, 0.0f));
}


#line 4851
struct FragmentOutput_0
{
    float4 lit_0 [[color(0)]];
    float4 reflectivity_0 [[color(1)]];
    float2 motion_0 [[color(2)]];
};


#line 4851
struct pixelInput_0
{
    float3 world_position_15 [[user(CRCBL_WORLD_POSITION)]];
    float3 world_normal_1 [[user(CRCBL_WORLD_NORMAL)]];
    float4 color_3 [[user(CRCBL_COLOR)]];
    [[flat]] uint material_13 [[user(CRCBL_MATERIAL)]];
    float2 uv_5 [[user(CRCBL_UV)]];
    float4 clip_position_1 [[user(CRCBL_CLIP_POSITION)]];
    float4 previous_clip_position_1 [[user(CRCBL_PREVIOUS_CLIP_POSITION)]];
    float3 world_tangent_1 [[user(CRCBL_WORLD_TANGENT)]];
    [[flat]] uint frame_4 [[user(CRCBL_FRAME)]];
};


#line 5298
[[fragment]] FragmentOutput_0 fragmentMain(pixelInput_0 _S263 [[stage_in]], bool front_facing_1 [[front_facing]], float4 position_5 [[position]], DrawConstants_0 constant* draw_3 [[buffer(3)]], uint device* visible_instances_3 [[buffer(5)]], GpuInstance_natural_0 device* instances_3 [[buffer(2)]], GpuMesh_0 device* meshes_3 [[buffer(4)]], FrameUniforms_natural_0 constant* frame_5 [[buffer(0)]], uint device* vertices_3 [[buffer(1)]], texture2d<float, access::sample> ambient_occlusion_3 [[texture(2)]], GpuMaterial_natural_0 device* materials_3 [[buffer(6)]], texture2d_array<float, access::sample> base_color_textures_3 [[texture(0)]], sampler base_color_sampler_3 [[sampler(0)]], texture2d_array<float, access::sample> normal_textures_3 [[texture(4)]], texture2d_array<float, access::sample> mro_textures_3 [[texture(8)]], texture2d_array<float, access::sample> emissive_textures_3 [[texture(9)]], uint device* cluster_lights_3 [[buffer(8)]], texture2d<float, access::sample> specular_dfg_3 [[texture(3)]], GpuLight_natural_0 device* lights_3 [[buffer(7)]], texture2d<float, access::sample> ltc_matrix_3 [[texture(5)]], depth2d<float, access::sample> shadow_atlas_3 [[texture(1)]], sampler shadow_sampler_3 [[sampler(1)]], texture2d<float, access::sample> contact_shadow_3 [[texture(6)]], GpuProbe_natural_0 device* probes_3 [[buffer(9)]], texture2d_array<float, access::sample> probe_visibility_3 [[texture(7)]])
{

#line 5298
    thread KernelContext_0 kernelContext_37;

#line 5298
    (&kernelContext_37)->draw_0 = draw_3;

#line 5298
    (&kernelContext_37)->visible_instances_0 = visible_instances_3;

#line 5298
    (&kernelContext_37)->instances_0 = instances_3;

#line 5298
    (&kernelContext_37)->meshes_0 = meshes_3;

#line 5298
    (&kernelContext_37)->frame_0 = frame_5;

#line 5298
    (&kernelContext_37)->vertices_0 = vertices_3;

#line 5298
    (&kernelContext_37)->ambient_occlusion_0 = ambient_occlusion_3;

#line 5298
    (&kernelContext_37)->materials_0 = materials_3;

#line 5298
    (&kernelContext_37)->base_color_textures_0 = base_color_textures_3;

#line 5298
    (&kernelContext_37)->base_color_sampler_0 = base_color_sampler_3;

#line 5298
    (&kernelContext_37)->normal_textures_0 = normal_textures_3;

#line 5298
    (&kernelContext_37)->mro_textures_0 = mro_textures_3;

#line 5298
    (&kernelContext_37)->emissive_textures_0 = emissive_textures_3;

#line 5298
    (&kernelContext_37)->cluster_lights_0 = cluster_lights_3;

#line 5298
    (&kernelContext_37)->specular_dfg_0 = specular_dfg_3;

#line 5298
    (&kernelContext_37)->lights_0 = lights_3;

#line 5298
    (&kernelContext_37)->ltc_matrix_0 = ltc_matrix_3;

#line 5298
    (&kernelContext_37)->shadow_atlas_0 = shadow_atlas_3;

#line 5298
    (&kernelContext_37)->shadow_sampler_0 = shadow_sampler_3;

#line 5298
    (&kernelContext_37)->contact_shadow_0 = contact_shadow_3;

#line 5298
    (&kernelContext_37)->probes_0 = probes_3;

#line 5298
    (&kernelContext_37)->probe_visibility_0 = probe_visibility_3;

#line 5310
    float3 vertex_normal_0 = normalize(_S263.world_normal_1);

#line 5315
    float2 motion_1 = motion_vector_0(_S263.clip_position_1, _S263.previous_clip_position_1);

#line 5331
    if((frame_5->ambient_0.w) >= 5.5f)
    {
        thread FragmentOutput_0 bent_0;

#line 5333
        float4 _S264 = occlusion_at_0(position_5.xy, &kernelContext_37);



        (&bent_0)->lit_0 = float4(_S264.yzw, 1.0f);


        (&bent_0)->reflectivity_0 = float4(0.0f, 0.0f, 0.0f, 1.0f);
        (&bent_0)->motion_0 = motion_1;
        return bent_0;
    }

    if((frame_5->ambient_0.w) >= 4.5f)
    {
        thread FragmentOutput_0 moved_0;
        (&moved_0)->lit_0 = float4(motion_1 * float2(8.0f)  + float2(0.5f) , 0.0f, 1.0f);


        (&moved_0)->reflectivity_0 = float4(0.0f, 0.0f, 0.0f, 1.0f);
        (&moved_0)->motion_0 = motion_1;
        return moved_0;
    }

#line 5387
    if((frame_5->ambient_0.w) >= 3.5f)
    {

#line 5387
        float4 _S265 = occlusion_at_0(position_5.xy, &kernelContext_37);


        float value_1 = _S265.x;

#line 5389
        thread FragmentOutput_0 occlusion_1;

#line 5398
        (&occlusion_1)->lit_0 = float4(value_1, value_1, value_1, 1.0f);


        (&occlusion_1)->reflectivity_0 = float4(0.0f, 0.0f, 0.0f, 1.0f);
        (&occlusion_1)->motion_0 = motion_1;
        return occlusion_1;
    }

    if((frame_5->ambient_0.w) >= 1.5f)
    {
        thread FragmentOutput_0 tint_0;



        (&tint_0)->lit_0 = float4(_S263.color_3.xyz, 1.0f);
        (&tint_0)->reflectivity_0 = float4(0.0f, 0.0f, 0.0f, 1.0f);
        (&tint_0)->motion_0 = motion_1;
        return tint_0;
    }

#line 5415
    thread GpuMaterial_natural_0 _S266 = (&kernelContext_37)->materials_0[_S263.material_13];

#line 5415
    float2 uv_6;

#line 5440
    if(((&_S266)->tiling_0) == 1U)
    {

#line 5440
        uv_6 = physical_tile_uv_0(_S263.world_position_15, vertex_normal_0, (&_S266)->tile_metres_0);

#line 5440
    }
    else
    {

#line 5440
        uv_6 = _S263.uv_5;

#line 5440
    }

#line 5440
    float4 _S267 = base_color_texel_0(&_S266, uv_6, &kernelContext_37);

#line 5462
    float4 albedo_1 = _S263.color_3 * float4((&_S266)->base_color_0)  * _S267;

#line 5476
    float _S268 = albedo_1.w;

#line 5476
    bool _S269 = alpha_masked_0(&_S266, _S268);

#line 5476
    if(_S269)
    {
        discard_fragment();

#line 5476
    }

#line 5476
    float3 _S270 = double_sided_normal_0(&_S266, vertex_normal_0, front_facing_1);

#line 5476
    uint _S271 = normal_layer_0(&_S266);

#line 5476
    thread VertexOutput_0 _S272;

#line 5476
    (&_S272)->position_3 = position_5;

#line 5476
    (&_S272)->world_position_1 = _S263.world_position_15;

#line 5476
    (&_S272)->world_normal_0 = _S263.world_normal_1;

#line 5476
    (&_S272)->color_2 = _S263.color_3;

#line 5476
    (&_S272)->material_6 = _S263.material_13;

#line 5476
    (&_S272)->uv_1 = _S263.uv_5;

#line 5476
    (&_S272)->clip_position_0 = _S263.clip_position_1;

#line 5476
    (&_S272)->previous_clip_position_0 = _S263.previous_clip_position_1;

#line 5476
    (&_S272)->world_tangent_0 = _S263.world_tangent_1;

#line 5476
    (&_S272)->frame_3 = _S263.frame_4;

#line 5476
    float3 _S273 = shading_normal_of_0(_S271, (&_S266)->normal_scale_0, &_S272, _S270, uv_6, &kernelContext_37);

#line 5495
    if((frame_5->ambient_0.w) >= 0.5f)
    {
        thread FragmentOutput_0 normals_0;

#line 5497
        float3 _S274 = float3(0.5f) ;

#line 5509
        (&normals_0)->lit_0 = float4(_S273 * _S274 + _S274, 1.0f);

#line 5515
        (&normals_0)->reflectivity_0 = float4(0.0f, 0.0f, 0.0f, 1.0f);
        (&normals_0)->motion_0 = motion_1;
        return normals_0;
    }

    float3 to_eye_1 = normalize((&kernelContext_37)->frame_0->camera_position_0.xyz - _S263.world_position_15);



    float3 _S275 = geometric_normal_of_0(_S263.world_position_15, _S270);

#line 5524
    float4 _S276 = mro_texel_0(&_S266, uv_6, &kernelContext_37);

#line 5524
    float4 _S277 = emissive_texel_0(&_S266, uv_6, &kernelContext_37);

#line 5524
    float _S278 = metallic_of_0(&_S266, _S276);

#line 5555
    float roughness_2 = clamp((&_S266)->roughness_0 * _S276.y, 0.04500000178813934f, 1.0f);
    float alpha_1 = roughness_2 * roughness_2;

#line 5589
    float _S279 = saturate(alpha_1 * alpha_1 + specular_aa_kernel_0(_S273));

#line 5595
    float3 _S280 = albedo_1.xyz;

#line 5595
    float3 f0_2 = mix(float3(0.03999999910593033f, 0.03999999910593033f, 0.03999999910593033f), _S280, float3(_S278) );
    float3 diffuse_albedo_0 = _S280 * float3((1.0f - _S278)) ;

#line 5602
    float _S281 = max(dot(_S273, to_eye_1), 0.00009999999747379f);

#line 5612
    float2 _S282 = position_5.xy;

#line 5612
    uint _S283 = froxel_of_0(_S282, (((float4(_S263.world_position_15, 1.0f)) * (matrix<float,int(4),int(4)> ((&kernelContext_37)->frame_0->view_proj_0.data_1[int(0)][int(0)], (&kernelContext_37)->frame_0->view_proj_0.data_1[int(1)][int(0)], (&kernelContext_37)->frame_0->view_proj_0.data_1[int(2)][int(0)], (&kernelContext_37)->frame_0->view_proj_0.data_1[int(3)][int(0)], (&kernelContext_37)->frame_0->view_proj_0.data_1[int(0)][int(1)], (&kernelContext_37)->frame_0->view_proj_0.data_1[int(1)][int(1)], (&kernelContext_37)->frame_0->view_proj_0.data_1[int(2)][int(1)], (&kernelContext_37)->frame_0->view_proj_0.data_1[int(3)][int(1)], (&kernelContext_37)->frame_0->view_proj_0.data_1[int(0)][int(2)], (&kernelContext_37)->frame_0->view_proj_0.data_1[int(1)][int(2)], (&kernelContext_37)->frame_0->view_proj_0.data_1[int(2)][int(2)], (&kernelContext_37)->frame_0->view_proj_0.data_1[int(3)][int(2)], (&kernelContext_37)->frame_0->view_proj_0.data_1[int(0)][int(3)], (&kernelContext_37)->frame_0->view_proj_0.data_1[int(1)][int(3)], (&kernelContext_37)->frame_0->view_proj_0.data_1[int(2)][int(3)], (&kernelContext_37)->frame_0->view_proj_0.data_1[int(3)][int(3)])))).w, &kernelContext_37);

#line 5612
    uint base_3 = _S283 * 17U;

#line 5617
    uint _S284 = min((&kernelContext_37)->cluster_lights_0[base_3], 16U);

#line 5617
    TableTap_0 _S285 = table_tap_0(_S281, roughness_2, &kernelContext_37);

#line 5617
    thread TableTap_0 _S286 = _S285;

#line 5617
    float2 _S287 = dfg_at_0(&_S286, &kernelContext_37);

#line 5626
    float _S288 = _S287.x;

#line 5626
    float _S289 = _S287.y;

#line 5626
    float3 _S290 = f0_2 * float3(_S288)  + float3(_S289) ;

#line 5632
    float3 _S291 = float3(0.0f, 0.0f, 0.0f);

#line 5632
    float3 sun_cascade_tint_0 = float3(1.0f, 1.0f, 1.0f);

#line 5632
    uint slot_0 = 0U;

#line 5632
    float3 direct_0 = _S291;

#line 5632
    float3 gloss_0 = _S291;

#line 5642
    for(;;)
    {

#line 5642
        if(slot_0 < _S284)
        {
        }
        else
        {

#line 5642
            break;
        }

#line 5642
        thread GpuLight_natural_0 _S292 = (&kernelContext_37)->lights_0[(&kernelContext_37)->cluster_lights_0[base_3 + 1U + slot_0]];

#line 5642
        uint _S293 = (&_S292)->kind_0;

#line 5651
        bool _S294 = ((&_S292)->kind_0) == 0U;

#line 5651
        float3 to_light_7;

#line 5651
        float reach_2;

#line 5651
        if(_S294)
        {

#line 5651
            to_light_7 = normalize((float4((&_S292)->direction_0) ).xyz);

#line 5651
            reach_2 = 1.0f;

#line 5651
        }
        else
        {


            if(_S293 == 3U)
            {

#line 5656
                float4 _S295 = float4((&_S292)->position_0) ;

#line 5664
                float3 offset_2 = _S295.xyz - _S263.world_position_15;
                float distance_3 = length(offset_2);

                float _S296 = range_window_0(distance_3, _S295.w);

#line 5667
                to_light_7 = offset_2 / float3(max(distance_3, 9.99999997475242708e-07f)) ;

#line 5667
                reach_2 = _S296;

#line 5656
            }
            else
            {

#line 5656
                float4 _S297 = float4((&_S292)->position_0) ;

#line 5671
                float3 offset_3 = _S297.xyz - _S263.world_position_15;
                float distance_4 = length(offset_3);
                float3 to_light_8 = offset_3 / float3(max(distance_4, 9.99999997475242708e-07f)) ;
                float reach_3 = punctual_falloff_0(distance_4, _S297.w);
                if(_S293 == 2U)
                {

#line 5675
                    float4 _S298 = float4((&_S292)->direction_0) ;

#line 5675
                    reach_2 = reach_3 * spot_cone_0(to_light_8, _S298.xyz, _S298.w, (&_S292)->cos_inner_0);

#line 5675
                }
                else
                {

#line 5675
                    reach_2 = reach_3;

#line 5675
                }

#line 5675
                to_light_7 = to_light_8;

#line 5656
            }

#line 5651
        }

#line 5684
        float n_dot_l_5 = dot(_S273, to_light_7);

#line 5684
        float3 specular_0;

#line 5684
        float diffuse_0;


        if(_S293 == 3U)
        {

#line 5697
            thread array<float3, int(4)> corners_2;

#line 5697
            rect_corners_0(&_S292, _S263.world_position_15, &corners_2);

            matrix<float,int(3),int(3)>  to_local_0 = ltc_shading_frame_0(_S273, to_eye_1, _S281);

#line 5699
            thread array<float3, int(4)> _S299 = corners_2;

#line 5699
            float _S300 = ltc_irradiance_0(to_local_0, &_S299);

#line 5699
            thread TableTap_0 _S301 = _S285;

#line 5699
            float4 _S302 = ltc_at_0(&_S301, &kernelContext_37);

            matrix<float,int(3),int(3)>  _S303 = (((to_local_0) * (ltc_transform_0(_S302))));

#line 5701
            thread array<float3, int(4)> _S304 = corners_2;

#line 5701
            float _S305 = ltc_irradiance_0(_S303, &_S304);
            float3 _S306 = float3(_S305)  * _S290;

#line 5702
            diffuse_0 = _S300;

#line 5702
            specular_0 = _S306;

#line 5687
        }
        else
        {

#line 5707
            float _S307 = max(n_dot_l_5, 0.0f);

#line 5714
            float3 half_vector_0 = normalize(to_light_7 + to_eye_1);

#line 5722
            float3 specular_1 = ggx_lobe_0(_S279, f0_2, _S307, _S281, max(dot(_S273, half_vector_0), 0.0f), max(dot(to_eye_1, half_vector_0), 0.0f)) * float3(_S307) ;

#line 5722
            diffuse_0 = _S307;

#line 5722
            specular_0 = specular_1;

#line 5687
        }

#line 5687
        float3 specular_2;

#line 5730
        if((((&_S292)->flags_3) & 1U) != 0U)
        {

#line 5730
            specular_2 = _S291;

#line 5730
        }
        else
        {

#line 5730
            specular_2 = specular_0;

#line 5730
        }

#line 5730
        float reach_4;

#line 5748
        if(_S294)
        {
            thread uint sun_cascade_0;
            thread float sun_fade_0;

#line 5751
            float _S308 = sun_visibility_0(_S263.world_position_15, to_light_7, n_dot_l_5, _S275, _S282, &sun_cascade_0, &sun_fade_0, &kernelContext_37);

#line 5751
            float _S309 = contact_at_0(_S282, &kernelContext_37);

#line 5760
            float _S310 = _S308 * _S309;

#line 5760
            sun_cascade_tint_0 = cascade_tint_0(sun_cascade_0, sun_fade_0);

#line 5760
            reach_4 = _S310;

#line 5748
        }
        else
        {

#line 5765
            if(_S293 == 1U)
            {

#line 5765
                uint _S311 = (&_S292)->shadow_tile_0;

#line 5777
                if(((&_S292)->shadow_tile_0) <= 8U)
                {

#line 5777
                    float _S312 = point_visibility_0(&_S292, _S311, _S263.world_position_15, to_light_7, n_dot_l_5, _S275, _S282, &kernelContext_37);

#line 5777
                    reach_4 = reach_2 * _S312;

#line 5777
                }
                else
                {

#line 5777
                    reach_4 = reach_2;

#line 5777
                }

#line 5765
            }
            else
            {

#line 5765
                uint _S313 = (&_S292)->shadow_tile_0;

#line 5783
                if(((&_S292)->shadow_tile_0) < 14U)
                {

#line 5783
                    float _S314 = spot_visibility_0(&_S292, _S313, _S263.world_position_15, to_light_7, n_dot_l_5, _S275, _S282, &kernelContext_37);

#line 5783
                    reach_4 = reach_2 * _S314;

#line 5783
                }
                else
                {

#line 5783
                    reach_4 = reach_2;

#line 5783
                }

#line 5765
            }

#line 5748
        }

#line 5791
        float3 _S315 = (float4((&_S292)->color_0) ).xyz;

#line 5791
        float3 direct_1 = direct_0 + _S315 * float3((diffuse_0 * reach_4)) ;
        float3 gloss_1 = gloss_0 + _S315 * (specular_2 * float3(reach_4) );

#line 5642
        slot_0 = slot_0 + 1U;

#line 5642
        direct_0 = direct_1;

#line 5642
        gloss_0 = gloss_1;

#line 5642
    }

#line 5806
    float3 gloss_2 = gloss_0 * specular_compensation_0(f0_2, _S288 + _S289);

#line 5806
    float4 _S316 = occlusion_at_0(_S282, &kernelContext_37);

#line 5825
    float occluded_0 = _S316.x;

#line 5834
    float3 bent_normal_0 = bent_normal_at_0(_S316, _S273);

#line 5857
    float3 _S317 = frame_5->ambient_0.xyz;

#line 5857
    float3 _S318 = sky_irradiance_0(bent_normal_0, &kernelContext_37);

#line 5857
    float3 _S319 = _S317 + _S318;

#line 5857
    float3 _S320 = probe_irradiance_0(_S263.world_position_15, bent_normal_0, &kernelContext_37);

#line 5913
    float3 lit_1 = diffuse_albedo_0 * ((_S319 + _S320) * (multi_bounce_occlusion_0(occluded_0, diffuse_albedo_0) * float3(_S276.x) ) + direct_0) + gloss_2;

#line 5913
    float3 _S321 = emissive_of_0(&_S266);

#line 5955
    float fog_survives_0 = fog_transmittance_0(fog_optical_depth_0((&kernelContext_37)->frame_0->fog_params_0.x, (&kernelContext_37)->frame_0->fog_params_0.y, (&kernelContext_37)->frame_0->camera_position_0.y - (&kernelContext_37)->frame_0->fog_params_0.z, _S263.world_position_15.y - (&kernelContext_37)->frame_0->fog_params_0.z, length((&kernelContext_37)->frame_0->camera_position_0.xyz - _S263.world_position_15)));
    float3 lit_2 = (lit_1 + _S321 * _S277.xyz) * float3(fog_survives_0)  + (&kernelContext_37)->frame_0->fog_color_0.xyz * float3((1.0f - fog_survives_0)) ;

    thread FragmentOutput_0 output_2;



    (&output_2)->lit_0 = float4(lit_2, _S268);


    (&output_2)->reflectivity_0 = float4(f0_2, floor(roughness_2 * 255.0f + 0.5f) / 255.0f);

    (&output_2)->motion_0 = motion_1;

#line 5975
    if((frame_5->ambient_0.w) <= -0.5f)
    {
        (&output_2)->lit_0 = float4(lit_2 * sun_cascade_tint_0, _S268);

#line 5984
        (&output_2)->reflectivity_0 = float4(0.0f, 0.0f, 0.0f, 1.0f);

#line 5975
    }

#line 5986
    return output_2;
}


#line 5986
struct pixelInput_1
{
    float3 world_position_16 [[user(CRCBL_WORLD_POSITION)]];
    float3 world_normal_2 [[user(CRCBL_WORLD_NORMAL)]];
    float4 color_4 [[user(CRCBL_COLOR)]];
    [[flat]] uint material_14 [[user(CRCBL_MATERIAL)]];
    float2 uv_7 [[user(CRCBL_UV)]];
    float4 clip_position_2 [[user(CRCBL_CLIP_POSITION)]];
    float4 previous_clip_position_2 [[user(CRCBL_PREVIOUS_CLIP_POSITION)]];
    float3 world_tangent_2 [[user(CRCBL_WORLD_TANGENT)]];
    [[flat]] uint frame_6 [[user(CRCBL_FRAME)]];
};


#line 6019
[[fragment]] void depthMaskedFragmentMain(pixelInput_1 _S322 [[stage_in]], float4 position_6 [[position]], DrawConstants_0 constant* draw_4 [[buffer(3)]], uint device* visible_instances_4 [[buffer(5)]], GpuInstance_natural_0 device* instances_4 [[buffer(2)]], GpuMesh_0 device* meshes_4 [[buffer(4)]], FrameUniforms_natural_0 constant* frame_7 [[buffer(0)]], uint device* vertices_4 [[buffer(1)]], texture2d<float, access::sample> ambient_occlusion_4 [[texture(2)]], GpuMaterial_natural_0 device* materials_4 [[buffer(6)]], texture2d_array<float, access::sample> base_color_textures_4 [[texture(0)]], sampler base_color_sampler_4 [[sampler(0)]], texture2d_array<float, access::sample> normal_textures_4 [[texture(4)]], texture2d_array<float, access::sample> mro_textures_4 [[texture(8)]], texture2d_array<float, access::sample> emissive_textures_4 [[texture(9)]], uint device* cluster_lights_4 [[buffer(8)]], texture2d<float, access::sample> specular_dfg_4 [[texture(3)]], GpuLight_natural_0 device* lights_4 [[buffer(7)]], texture2d<float, access::sample> ltc_matrix_4 [[texture(5)]], depth2d<float, access::sample> shadow_atlas_4 [[texture(1)]], sampler shadow_sampler_4 [[sampler(1)]], texture2d<float, access::sample> contact_shadow_4 [[texture(6)]], GpuProbe_natural_0 device* probes_4 [[buffer(9)]], texture2d_array<float, access::sample> probe_visibility_4 [[texture(7)]])
{

#line 6019
    thread KernelContext_0 kernelContext_38;

#line 6019
    (&kernelContext_38)->draw_0 = draw_4;

#line 6019
    (&kernelContext_38)->visible_instances_0 = visible_instances_4;

#line 6019
    (&kernelContext_38)->instances_0 = instances_4;

#line 6019
    (&kernelContext_38)->meshes_0 = meshes_4;

#line 6019
    (&kernelContext_38)->frame_0 = frame_7;

#line 6019
    (&kernelContext_38)->vertices_0 = vertices_4;

#line 6019
    (&kernelContext_38)->ambient_occlusion_0 = ambient_occlusion_4;

#line 6019
    (&kernelContext_38)->materials_0 = materials_4;

#line 6019
    (&kernelContext_38)->base_color_textures_0 = base_color_textures_4;

#line 6019
    (&kernelContext_38)->base_color_sampler_0 = base_color_sampler_4;

#line 6019
    (&kernelContext_38)->normal_textures_0 = normal_textures_4;

#line 6019
    (&kernelContext_38)->mro_textures_0 = mro_textures_4;

#line 6019
    (&kernelContext_38)->emissive_textures_0 = emissive_textures_4;

#line 6019
    (&kernelContext_38)->cluster_lights_0 = cluster_lights_4;

#line 6019
    (&kernelContext_38)->specular_dfg_0 = specular_dfg_4;

#line 6019
    (&kernelContext_38)->lights_0 = lights_4;

#line 6019
    (&kernelContext_38)->ltc_matrix_0 = ltc_matrix_4;

#line 6019
    (&kernelContext_38)->shadow_atlas_0 = shadow_atlas_4;

#line 6019
    (&kernelContext_38)->shadow_sampler_0 = shadow_sampler_4;

#line 6019
    (&kernelContext_38)->contact_shadow_0 = contact_shadow_4;

#line 6019
    (&kernelContext_38)->probes_0 = probes_4;

#line 6019
    (&kernelContext_38)->probe_visibility_0 = probe_visibility_4;

#line 6019
    thread GpuMaterial_natural_0 _S323 = materials_4[_S322.material_14];

#line 6019
    float2 uv_8;

#line 6028
    if(((&_S323)->tiling_0) == 1U)
    {

#line 6028
        uv_8 = physical_tile_uv_0(_S322.world_position_16, normalize(_S322.world_normal_2), (&_S323)->tile_metres_0);

#line 6028
    }
    else
    {

#line 6028
        uv_8 = _S322.uv_7;

#line 6028
    }

#line 6028
    float4 _S324 = base_color_texel_0(&_S323, uv_8, &kernelContext_38);

#line 6028
    bool _S325 = alpha_masked_0(&_S323, _S322.color_4.w * (float4((&_S323)->base_color_0) ).w * _S324.w);

#line 6037
    if(_S325)
    {
        discard_fragment();

#line 6037
    }



    return;
}


#line 6071
struct RsmOutput_0
{
    float4 albedo_2 [[color(0)]];
    float4 normal_14 [[color(1)]];
    float4 world_0 [[color(2)]];
};


#line 6071
struct pixelInput_2
{
    float3 world_position_17 [[user(CRCBL_WORLD_POSITION)]];
    float3 world_normal_3 [[user(CRCBL_WORLD_NORMAL)]];
    float4 color_5 [[user(CRCBL_COLOR)]];
    [[flat]] uint material_15 [[user(CRCBL_MATERIAL)]];
    float2 uv_9 [[user(CRCBL_UV)]];
    float4 clip_position_3 [[user(CRCBL_CLIP_POSITION)]];
    float4 previous_clip_position_3 [[user(CRCBL_PREVIOUS_CLIP_POSITION)]];
    float3 world_tangent_3 [[user(CRCBL_WORLD_TANGENT)]];
    [[flat]] uint frame_8 [[user(CRCBL_FRAME)]];
};


#line 6114
[[fragment]] RsmOutput_0 rsmFragmentMain(pixelInput_2 _S326 [[stage_in]], bool front_facing_2 [[front_facing]], float4 position_7 [[position]], DrawConstants_0 constant* draw_5 [[buffer(3)]], uint device* visible_instances_5 [[buffer(5)]], GpuInstance_natural_0 device* instances_5 [[buffer(2)]], GpuMesh_0 device* meshes_5 [[buffer(4)]], FrameUniforms_natural_0 constant* frame_9 [[buffer(0)]], uint device* vertices_5 [[buffer(1)]], texture2d<float, access::sample> ambient_occlusion_5 [[texture(2)]], GpuMaterial_natural_0 device* materials_5 [[buffer(6)]], texture2d_array<float, access::sample> base_color_textures_5 [[texture(0)]], sampler base_color_sampler_5 [[sampler(0)]], texture2d_array<float, access::sample> normal_textures_5 [[texture(4)]], texture2d_array<float, access::sample> mro_textures_5 [[texture(8)]], texture2d_array<float, access::sample> emissive_textures_5 [[texture(9)]], uint device* cluster_lights_5 [[buffer(8)]], texture2d<float, access::sample> specular_dfg_5 [[texture(3)]], GpuLight_natural_0 device* lights_5 [[buffer(7)]], texture2d<float, access::sample> ltc_matrix_5 [[texture(5)]], depth2d<float, access::sample> shadow_atlas_5 [[texture(1)]], sampler shadow_sampler_5 [[sampler(1)]], texture2d<float, access::sample> contact_shadow_5 [[texture(6)]], GpuProbe_natural_0 device* probes_5 [[buffer(9)]], texture2d_array<float, access::sample> probe_visibility_5 [[texture(7)]])
{

#line 6114
    thread KernelContext_0 kernelContext_39;

#line 6114
    (&kernelContext_39)->draw_0 = draw_5;

#line 6114
    (&kernelContext_39)->visible_instances_0 = visible_instances_5;

#line 6114
    (&kernelContext_39)->instances_0 = instances_5;

#line 6114
    (&kernelContext_39)->meshes_0 = meshes_5;

#line 6114
    (&kernelContext_39)->frame_0 = frame_9;

#line 6114
    (&kernelContext_39)->vertices_0 = vertices_5;

#line 6114
    (&kernelContext_39)->ambient_occlusion_0 = ambient_occlusion_5;

#line 6114
    (&kernelContext_39)->materials_0 = materials_5;

#line 6114
    (&kernelContext_39)->base_color_textures_0 = base_color_textures_5;

#line 6114
    (&kernelContext_39)->base_color_sampler_0 = base_color_sampler_5;

#line 6114
    (&kernelContext_39)->normal_textures_0 = normal_textures_5;

#line 6114
    (&kernelContext_39)->mro_textures_0 = mro_textures_5;

#line 6114
    (&kernelContext_39)->emissive_textures_0 = emissive_textures_5;

#line 6114
    (&kernelContext_39)->cluster_lights_0 = cluster_lights_5;

#line 6114
    (&kernelContext_39)->specular_dfg_0 = specular_dfg_5;

#line 6114
    (&kernelContext_39)->lights_0 = lights_5;

#line 6114
    (&kernelContext_39)->ltc_matrix_0 = ltc_matrix_5;

#line 6114
    (&kernelContext_39)->shadow_atlas_0 = shadow_atlas_5;

#line 6114
    (&kernelContext_39)->shadow_sampler_0 = shadow_sampler_5;

#line 6114
    (&kernelContext_39)->contact_shadow_0 = contact_shadow_5;

#line 6114
    (&kernelContext_39)->probes_0 = probes_5;

#line 6114
    (&kernelContext_39)->probe_visibility_0 = probe_visibility_5;

#line 6119
    float3 vertex_normal_1 = normalize(_S326.world_normal_3);

#line 6119
    thread GpuMaterial_natural_0 _S327 = materials_5[_S326.material_15];

#line 6119
    float2 uv_10;

#line 6126
    if(((&_S327)->tiling_0) == 1U)
    {

#line 6126
        uv_10 = physical_tile_uv_0(_S326.world_position_17, vertex_normal_1, (&_S327)->tile_metres_0);

#line 6126
    }
    else
    {

#line 6126
        uv_10 = _S326.uv_9;

#line 6126
    }

#line 6126
    float4 _S328 = base_color_texel_0(&_S327, uv_10, &kernelContext_39);

#line 6131
    float4 albedo_3 = _S326.color_5 * float4((&_S327)->base_color_0)  * _S328;

#line 6131
    bool _S329 = alpha_masked_0(&_S327, albedo_3.w);

#line 6137
    if(_S329)
    {
        discard_fragment();

#line 6137
    }

#line 6142
    thread RsmOutput_0 written_0;

#line 6152
    float3 _S330 = albedo_3.xyz;

#line 6152
    float4 _S331 = mro_texel_0(&_S327, uv_10, &kernelContext_39);

#line 6152
    float _S332 = metallic_of_0(&_S327, _S331);

#line 6151
    (&written_0)->albedo_2 = float4(_S330 * float3((1.0f - _S332)) , 1.0f);

#line 6151
    float3 _S333 = double_sided_normal_0(&_S327, vertex_normal_1, front_facing_2);

#line 6151
    float3 _S334 = float3(0.5f) ;

#line 6158
    (&written_0)->normal_14 = float4(_S333 * _S334 + _S334, 1.0f);

    (&written_0)->world_0 = float4(_S326.world_position_17, 1.0f);
    return written_0;
}


#line 6161
struct vertexMain_Result_0
{
    float4 position_8 [[position]];
    float3 world_position_18 [[user(CRCBL_WORLD_POSITION)]];
    float3 world_normal_4 [[user(CRCBL_WORLD_NORMAL)]];
    float4 color_6 [[user(CRCBL_COLOR)]];
    uint material_16 [[user(CRCBL_MATERIAL)]];
    float2 uv_11 [[user(CRCBL_UV)]];
    float4 clip_position_4 [[user(CRCBL_CLIP_POSITION)]];
    float4 previous_clip_position_4 [[user(CRCBL_PREVIOUS_CLIP_POSITION)]];
    float3 world_tangent_4 [[user(CRCBL_WORLD_TANGENT)]];
    uint frame_10 [[user(CRCBL_FRAME)]];
};


#line 6161
[[vertex]] vertexMain_Result_0 vertexMain(uint index_8 [[vertex_id]], uint instance_id_1 [[instance_id]], DrawConstants_0 constant* draw_6 [[buffer(3)]], uint device* visible_instances_6 [[buffer(5)]], GpuInstance_natural_0 device* instances_6 [[buffer(2)]], GpuMesh_0 device* meshes_6 [[buffer(4)]], FrameUniforms_natural_0 constant* frame_11 [[buffer(0)]], uint device* vertices_6 [[buffer(1)]], texture2d<float, access::sample> ambient_occlusion_6 [[texture(2)]], GpuMaterial_natural_0 device* materials_6 [[buffer(6)]], texture2d_array<float, access::sample> base_color_textures_6 [[texture(0)]], sampler base_color_sampler_6 [[sampler(0)]], texture2d_array<float, access::sample> normal_textures_6 [[texture(4)]], texture2d_array<float, access::sample> mro_textures_6 [[texture(8)]], texture2d_array<float, access::sample> emissive_textures_6 [[texture(9)]], uint device* cluster_lights_6 [[buffer(8)]], texture2d<float, access::sample> specular_dfg_6 [[texture(3)]], GpuLight_natural_0 device* lights_6 [[buffer(7)]], texture2d<float, access::sample> ltc_matrix_6 [[texture(5)]], depth2d<float, access::sample> shadow_atlas_6 [[texture(1)]], sampler shadow_sampler_6 [[sampler(1)]], texture2d<float, access::sample> contact_shadow_6 [[texture(6)]], GpuProbe_natural_0 device* probes_6 [[buffer(9)]], texture2d_array<float, access::sample> probe_visibility_6 [[texture(7)]])
{

#line 6161
    thread KernelContext_0 kernelContext_40;

#line 6161
    (&kernelContext_40)->draw_0 = draw_6;

#line 6161
    (&kernelContext_40)->visible_instances_0 = visible_instances_6;

#line 6161
    (&kernelContext_40)->instances_0 = instances_6;

#line 6161
    (&kernelContext_40)->meshes_0 = meshes_6;

#line 6161
    (&kernelContext_40)->frame_0 = frame_11;

#line 6161
    (&kernelContext_40)->vertices_0 = vertices_6;

#line 6161
    (&kernelContext_40)->ambient_occlusion_0 = ambient_occlusion_6;

#line 6161
    (&kernelContext_40)->materials_0 = materials_6;

#line 6161
    (&kernelContext_40)->base_color_textures_0 = base_color_textures_6;

#line 6161
    (&kernelContext_40)->base_color_sampler_0 = base_color_sampler_6;

#line 6161
    (&kernelContext_40)->normal_textures_0 = normal_textures_6;

#line 6161
    (&kernelContext_40)->mro_textures_0 = mro_textures_6;

#line 6161
    (&kernelContext_40)->emissive_textures_0 = emissive_textures_6;

#line 6161
    (&kernelContext_40)->cluster_lights_0 = cluster_lights_6;

#line 6161
    (&kernelContext_40)->specular_dfg_0 = specular_dfg_6;

#line 6161
    (&kernelContext_40)->lights_0 = lights_6;

#line 6161
    (&kernelContext_40)->ltc_matrix_0 = ltc_matrix_6;

#line 6161
    (&kernelContext_40)->shadow_atlas_0 = shadow_atlas_6;

#line 6161
    (&kernelContext_40)->shadow_sampler_0 = shadow_sampler_6;

#line 6161
    (&kernelContext_40)->contact_shadow_0 = contact_shadow_6;

#line 6161
    (&kernelContext_40)->probes_0 = probes_6;

#line 6161
    (&kernelContext_40)->probe_visibility_0 = probe_visibility_6;

#line 6161
    GpuInstance_natural_0 device* _S335 = instances_6+visible_instances_6[draw_6->base_0 + instance_id_1];

#line 2137
    GpuMesh_0 mesh_3 = meshes_6[draw_6->mesh_0];

#line 2145
    bool _S336 = ((_S335->flags_0) & 2U) != 0U;

#line 2145
    uint base_vertex_3;
    if(_S336)
    {

#line 2146
        base_vertex_3 = _S335->base_vertex_0;

#line 2146
    }
    else
    {

#line 2146
        base_vertex_3 = mesh_3.base_vertex_1;

#line 2146
    }

#line 2146
    MeshVertex_0 _S337 = load_vertex_0(index_8 + base_vertex_3, float4(mesh_3.uv_scale_u_0, mesh_3.uv_scale_v_0, mesh_3.uv_offset_u_0, mesh_3.uv_offset_v_0), &kernelContext_40);

#line 2146
    uint previous_base_0;

#line 2159
    if(_S336)
    {

#line 2159
        previous_base_0 = _S335->previous_base_vertex_0;

#line 2159
    }
    else
    {

#line 2159
        previous_base_0 = base_vertex_3;

#line 2159
    }

#line 2159
    float3 _S338 = load_position_0(index_8 + previous_base_0, &kernelContext_40);

#line 2159
    matrix<float,int(4),int(4)>  _S339 = matrix<float,int(4),int(4)> (_S335->transform_0.data_0[int(0)][int(0)], _S335->transform_0.data_0[int(1)][int(0)], _S335->transform_0.data_0[int(2)][int(0)], _S335->transform_0.data_0[int(3)][int(0)], _S335->transform_0.data_0[int(0)][int(1)], _S335->transform_0.data_0[int(1)][int(1)], _S335->transform_0.data_0[int(2)][int(1)], _S335->transform_0.data_0[int(3)][int(1)], _S335->transform_0.data_0[int(0)][int(2)], _S335->transform_0.data_0[int(1)][int(2)], _S335->transform_0.data_0[int(2)][int(2)], _S335->transform_0.data_0[int(3)][int(2)], _S335->transform_0.data_0[int(0)][int(3)], _S335->transform_0.data_0[int(1)][int(3)], _S335->transform_0.data_0[int(2)][int(3)], _S335->transform_0.data_0[int(3)][int(3)]);



    float4 world_1 = (((float4(_S337.position_1, 1.0f)) * (_S339)));

    thread VertexOutput_0 output_3;
    (&output_3)->position_3 = (((world_1) * (matrix<float,int(4),int(4)> ((&kernelContext_40)->frame_0->view_proj_0.data_1[int(0)][int(0)], (&kernelContext_40)->frame_0->view_proj_0.data_1[int(1)][int(0)], (&kernelContext_40)->frame_0->view_proj_0.data_1[int(2)][int(0)], (&kernelContext_40)->frame_0->view_proj_0.data_1[int(3)][int(0)], (&kernelContext_40)->frame_0->view_proj_0.data_1[int(0)][int(1)], (&kernelContext_40)->frame_0->view_proj_0.data_1[int(1)][int(1)], (&kernelContext_40)->frame_0->view_proj_0.data_1[int(2)][int(1)], (&kernelContext_40)->frame_0->view_proj_0.data_1[int(3)][int(1)], (&kernelContext_40)->frame_0->view_proj_0.data_1[int(0)][int(2)], (&kernelContext_40)->frame_0->view_proj_0.data_1[int(1)][int(2)], (&kernelContext_40)->frame_0->view_proj_0.data_1[int(2)][int(2)], (&kernelContext_40)->frame_0->view_proj_0.data_1[int(3)][int(2)], (&kernelContext_40)->frame_0->view_proj_0.data_1[int(0)][int(3)], (&kernelContext_40)->frame_0->view_proj_0.data_1[int(1)][int(3)], (&kernelContext_40)->frame_0->view_proj_0.data_1[int(2)][int(3)], (&kernelContext_40)->frame_0->view_proj_0.data_1[int(3)][int(3)]))));
    (&output_3)->world_position_1 = world_1.xyz;

#line 2173
    matrix<float,int(3),int(3)>  _S340 = matrix<float,int(3),int(3)> (_S339[int(0)].xyz, _S339[int(1)].xyz, _S339[int(2)].xyz);

#line 2173
    (&output_3)->world_normal_0 = (((_S337.basis_1.normal_0) * (normal_basis_0(_S340))));

#line 2179
    (&output_3)->world_tangent_0 = (((_S337.basis_1.tangent_1) * (_S340)));

#line 2179
    thread TangentFrame_0 _S341 = _S337.basis_1;

#line 2179
    uint _S342 = frame_word_0(mesh_3.flags_1, &_S341);
    (&output_3)->frame_3 = _S342;

#line 2180
    float4 _S343;

#line 2187
    if(((&kernelContext_40)->frame_0->ambient_0.w) >= 1.5f)
    {

#line 2187
        _S343 = float4(0.44999998807907104f, 0.44999998807907104f, 0.47999998927116394f, 1.0f);

#line 2187
    }
    else
    {

#line 2187
        _S343 = _S337.color_1;

#line 2187
    }

#line 2186
    (&output_3)->color_2 = _S343;

#line 2193
    (&output_3)->material_6 = _S335->material_0;
    (&output_3)->uv_1 = _S337.uv0_0;

#line 2200
    (&output_3)->clip_position_0 = (&output_3)->position_3;
    (&output_3)->previous_clip_position_0 = ((((((float4(_S338, 1.0f)) * (matrix<float,int(4),int(4)> (_S335->previous_transform_0.data_0[int(0)][int(0)], _S335->previous_transform_0.data_0[int(1)][int(0)], _S335->previous_transform_0.data_0[int(2)][int(0)], _S335->previous_transform_0.data_0[int(3)][int(0)], _S335->previous_transform_0.data_0[int(0)][int(1)], _S335->previous_transform_0.data_0[int(1)][int(1)], _S335->previous_transform_0.data_0[int(2)][int(1)], _S335->previous_transform_0.data_0[int(3)][int(1)], _S335->previous_transform_0.data_0[int(0)][int(2)], _S335->previous_transform_0.data_0[int(1)][int(2)], _S335->previous_transform_0.data_0[int(2)][int(2)], _S335->previous_transform_0.data_0[int(3)][int(2)], _S335->previous_transform_0.data_0[int(0)][int(3)], _S335->previous_transform_0.data_0[int(1)][int(3)], _S335->previous_transform_0.data_0[int(2)][int(3)], _S335->previous_transform_0.data_0[int(3)][int(3)]))))) * (matrix<float,int(4),int(4)> ((&kernelContext_40)->frame_0->previous_view_proj_0.data_1[int(0)][int(0)], (&kernelContext_40)->frame_0->previous_view_proj_0.data_1[int(1)][int(0)], (&kernelContext_40)->frame_0->previous_view_proj_0.data_1[int(2)][int(0)], (&kernelContext_40)->frame_0->previous_view_proj_0.data_1[int(3)][int(0)], (&kernelContext_40)->frame_0->previous_view_proj_0.data_1[int(0)][int(1)], (&kernelContext_40)->frame_0->previous_view_proj_0.data_1[int(1)][int(1)], (&kernelContext_40)->frame_0->previous_view_proj_0.data_1[int(2)][int(1)], (&kernelContext_40)->frame_0->previous_view_proj_0.data_1[int(3)][int(1)], (&kernelContext_40)->frame_0->previous_view_proj_0.data_1[int(0)][int(2)], (&kernelContext_40)->frame_0->previous_view_proj_0.data_1[int(1)][int(2)], (&kernelContext_40)->frame_0->previous_view_proj_0.data_1[int(2)][int(2)], (&kernelContext_40)->frame_0->previous_view_proj_0.data_1[int(3)][int(2)], (&kernelContext_40)->frame_0->previous_view_proj_0.data_1[int(0)][int(3)], (&kernelContext_40)->frame_0->previous_view_proj_0.data_1[int(1)][int(3)], (&kernelContext_40)->frame_0->previous_view_proj_0.data_1[int(2)][int(3)], (&kernelContext_40)->frame_0->previous_view_proj_0.data_1[int(3)][int(3)]))));


    VertexOutput_0 _S344 = output_3;

#line 2204
    thread vertexMain_Result_0 _S345;

#line 2204
    (&_S345)->position_8 = _S344.position_3;

#line 2204
    (&_S345)->world_position_18 = _S344.world_position_1;

#line 2204
    (&_S345)->world_normal_4 = _S344.world_normal_0;

#line 2204
    (&_S345)->color_6 = _S344.color_2;

#line 2204
    (&_S345)->material_16 = _S344.material_6;

#line 2204
    (&_S345)->uv_11 = _S344.uv_1;

#line 2204
    (&_S345)->clip_position_4 = _S344.clip_position_0;

#line 2204
    (&_S345)->previous_clip_position_4 = _S344.previous_clip_position_0;

#line 2204
    (&_S345)->world_tangent_4 = _S344.world_tangent_0;

#line 2204
    (&_S345)->frame_10 = _S344.frame_3;

#line 2204
    return _S345;
}

