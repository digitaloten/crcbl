#include <metal_stdlib>
#include <metal_math>
#include <metal_texture>
using namespace metal;

#line 71 "shaders/cmaa2_shapes.slang"
struct Cmaa2Params_0
{
    uint viewport_x_0;
    uint viewport_y_0;
};


#line 271
struct KernelContext_0
{
    Cmaa2Params_0 constant* params_0;
    uint device* edges_0;
    texture2d<float, access::sample> source_0;
    atomic<uint> device* accum_0;
};


#line 159
uint edge_at_0(uint x_0, uint y_0, KernelContext_0 thread* kernelContext_0)
{

#line 159
    bool _S1;

    if(x_0 >= (kernelContext_0->params_0->viewport_x_0))
    {

#line 161
        _S1 = true;

#line 161
    }
    else
    {

#line 161
        _S1 = y_0 >= (kernelContext_0->params_0->viewport_y_0);

#line 161
    }

#line 161
    if(_S1)
    {
        return 0U;
    }
    return *(kernelContext_0->edges_0+(y_0 * kernelContext_0->params_0->viewport_x_0 + x_0));
}


#line 159
uint edge_at_1(uint x_1, uint y_1, KernelContext_0 thread* kernelContext_1)
{

#line 159
    bool _S2;

    if(x_1 >= (kernelContext_1->params_0->viewport_x_0))
    {

#line 161
        _S2 = true;

#line 161
    }
    else
    {

#line 161
        _S2 = y_1 >= (kernelContext_1->params_0->viewport_y_0);

#line 161
    }

#line 161
    if(_S2)
    {
        return 0U;
    }
    return *(kernelContext_1->edges_0+(y_1 * kernelContext_1->params_0->viewport_x_0 + x_1));
}


#line 176
int horizontal_turn_0(uint column_0, uint row_0, KernelContext_0 thread* kernelContext_2)
{

#line 176
    uint _S3 = edge_at_1(column_0, row_0, kernelContext_2);

    bool below_0 = (_S3 & 1U) != 0U;

#line 178
    bool above_0;
    if(row_0 > 0U)
    {

#line 179
        uint _S4 = edge_at_1(column_0, row_0 - 1U, kernelContext_2);

#line 179
        above_0 = (_S4 & 1U) != 0U;

#line 179
    }
    else
    {

#line 179
        above_0 = false;

#line 179
    }

#line 179
    bool _S5;
    if(above_0)
    {

#line 180
        _S5 = !below_0;

#line 180
    }
    else
    {

#line 180
        _S5 = false;

#line 180
    }

#line 180
    if(_S5)
    {
        return int(1);
    }
    if(below_0)
    {

#line 184
        above_0 = !above_0;

#line 184
    }
    else
    {

#line 184
        above_0 = false;

#line 184
    }

#line 184
    if(above_0)
    {
        return int(-1);
    }
    return int(0);
}


#line 220
float boundary_height_0(float t_0, int start_0, int end_0, bool u_shape_0)
{
    if(u_shape_0)
    {
        return -0.5f * float(start_0) * (2.0f * abs(t_0 - 0.5f)) * 0.5f;
    }
    return -0.5f * float(start_0) * (1.0f - t_0) - 0.5f * float(end_0) * t_0;
}


#line 250
void accumulate_0(uint target_0, uint from_0, float share_0, KernelContext_0 thread* kernelContext_3)
{
    if(share_0 < 0.001953125f)
    {
        return;
    }
    uint pixels_0 = kernelContext_3->params_0->viewport_x_0 * kernelContext_3->params_0->viewport_y_0;

#line 256
    bool _S6;
    if(target_0 >= pixels_0)
    {

#line 257
        _S6 = true;

#line 257
    }
    else
    {

#line 257
        _S6 = from_0 >= pixels_0;

#line 257
    }

#line 257
    if(_S6)
    {
        return;
    }

    uint width_0 = kernelContext_3->params_0->viewport_x_0;
    uint _S7 = from_0 % kernelContext_3->params_0->viewport_x_0;

#line 263
    int _S8 = int(_S7);

#line 263
    uint _S9 = from_0 / width_0;



    int3 _S10 = int3(int2(_S8, int(_S9)), int(0));

#line 267
    float3 color_0 = saturate(((kernelContext_3->source_0).read(vec<uint,2>(((_S10)).xy), uint(((_S10)).z))).xyz);
    uint weight_0 = uint(min(share_0, 0.5f) * 1.048576e+06f);
    float scaled_0 = float(weight_0);

    uint _S11 = target_0 * 4U;

#line 271
    uint _S12 = atomic_fetch_add_explicit(kernelContext_3->accum_0+_S11, uint(color_0.x * scaled_0), memory_order_relaxed);
    uint _S13 = atomic_fetch_add_explicit(kernelContext_3->accum_0+(_S11 + 1U), uint(color_0.y * scaled_0), memory_order_relaxed);
    uint _S14 = atomic_fetch_add_explicit(kernelContext_3->accum_0+(_S11 + 2U), uint(color_0.z * scaled_0), memory_order_relaxed);
    uint _S15 = atomic_fetch_add_explicit(kernelContext_3->accum_0+(_S11 + 3U), weight_0, memory_order_relaxed);
    return;
}


#line 299
void blend_line_0(uint first_0, uint stride_0, uint len_0, uint offset_0, int start_1, int end_1, KernelContext_0 thread* kernelContext_4)
{

#line 299
    bool _S16;

    if(start_1 == int(0))
    {

#line 301
        _S16 = true;

#line 301
    }
    else
    {

#line 301
        _S16 = end_1 == int(0);

#line 301
    }

#line 301
    if(_S16)
    {
        return;
    }
    float _S17 = 1.0f / float(len_0);
    bool _S18 = start_1 == end_1;

#line 306
    uint i_0 = 0U;
    for(;;)
    {

#line 307
        if(i_0 < len_0)
        {
        }
        else
        {

#line 307
            break;
        }

#line 314
        float near_0 = boundary_height_0(float(i_0) * _S17, start_1, end_1, _S18);
        uint _S19 = i_0 + 1U;

#line 315
        float far_0 = boundary_height_0(float(_S19) * _S17, start_1, end_1, _S18);

#line 322
        if(near_0 <= 0.0f)
        {

#line 322
            _S16 = far_0 <= 0.0f;

#line 322
        }
        else
        {

#line 322
            _S16 = false;

#line 322
        }

#line 322
        float below_1;

#line 322
        float above_1;

#line 322
        if(_S16)
        {

#line 322
            below_1 = -0.5f * (near_0 + far_0);

#line 322
            above_1 = 0.0f;

#line 322
        }
        else
        {

#line 322
            bool _S20;

#line 327
            if(near_0 >= 0.0f)
            {

#line 327
                _S20 = far_0 >= 0.0f;

#line 327
            }
            else
            {

#line 327
                _S20 = false;

#line 327
            }

#line 327
            if(_S20)
            {

                float _S21 = 0.5f * (near_0 + far_0);

#line 330
                below_1 = 0.0f;

#line 330
                above_1 = _S21;

#line 327
            }
            else
            {

#line 334
                float crossing_0 = near_0 / (near_0 - far_0);
                float _S22 = 0.5f * abs(min(near_0, far_0));

#line 335
                if(near_0 < 0.0f)
                {

#line 335
                    below_1 = crossing_0;

#line 335
                }
                else
                {

#line 335
                    below_1 = 1.0f - crossing_0;

#line 335
                }

#line 335
                float _S23 = _S22 * below_1;
                float _S24 = 0.5f * max(near_0, far_0);

#line 336
                if(near_0 > 0.0f)
                {

#line 336
                    above_1 = crossing_0;

#line 336
                }
                else
                {

#line 336
                    above_1 = 1.0f - crossing_0;

#line 336
                }

#line 336
                float _S25 = _S24 * above_1;

#line 336
                below_1 = _S23;

#line 336
                above_1 = _S25;

#line 327
            }

#line 322
        }

#line 339
        uint pixel_0 = first_0 + i_0 * stride_0;

#line 348
        uint _S26 = pixel_0 - offset_0;

#line 348
        accumulate_0(_S26, pixel_0, below_1, kernelContext_4);

#line 348
        accumulate_0(pixel_0, _S26, above_1, kernelContext_4);

#line 307
        i_0 = _S19;

#line 307
    }

#line 351
    return;
}


#line 194
int vertical_turn_0(uint column_1, uint row_1, KernelContext_0 thread* kernelContext_5)
{

#line 194
    uint _S27 = edge_at_1(column_1, row_1, kernelContext_5);

    bool right_0 = (_S27 & 2U) != 0U;

#line 196
    bool left_0;
    if(column_1 > 0U)
    {

#line 197
        uint _S28 = edge_at_1(column_1 - 1U, row_1, kernelContext_5);

#line 197
        left_0 = (_S28 & 2U) != 0U;

#line 197
    }
    else
    {

#line 197
        left_0 = false;

#line 197
    }

#line 197
    bool _S29;
    if(left_0)
    {

#line 198
        _S29 = !right_0;

#line 198
    }
    else
    {

#line 198
        _S29 = false;

#line 198
    }

#line 198
    if(_S29)
    {
        return int(1);
    }
    if(right_0)
    {

#line 202
        left_0 = !left_0;

#line 202
    }
    else
    {

#line 202
        left_0 = false;

#line 202
    }

#line 202
    if(left_0)
    {
        return int(-1);
    }
    return int(0);
}


#line 363
[[kernel]] void shapesMain(uint3 thread_0 [[thread_position_in_grid]], Cmaa2Params_0 constant* params_1 [[buffer(0)]], uint device* edges_1 [[buffer(1)]], texture2d<float, access::sample> source_1 [[texture(0)]], atomic<uint> device* accum_1 [[buffer(2)]])
{

#line 363
    bool _S30;

#line 363
    bool _S31;

#line 363
    thread KernelContext_0 kernelContext_6;

#line 363
    (&kernelContext_6)->params_0 = params_1;

#line 363
    (&kernelContext_6)->edges_0 = edges_1;

#line 363
    (&kernelContext_6)->source_0 = source_1;

#line 363
    (&kernelContext_6)->accum_0 = accum_1;

    uint index_0 = thread_0.x;
    uint width_1 = params_1->viewport_x_0;

    if(index_0 >= (params_1->viewport_x_0 * params_1->viewport_y_0))
    {
        return;
    }

    uint device* _S32 = (&kernelContext_6)->edges_0+index_0;

#line 373
    uint own_0 = *_S32;
    if((*_S32) == 0U)
    {
        return;
    }

    uint x_2 = index_0 % width_1;
    uint y_2 = index_0 / width_1;

#line 380
    bool _S33;



    if((own_0 & 2U) != 0U)
    {

#line 384
        uint _S34 = edge_at_0(x_2 - 1U, y_2, &kernelContext_6);

#line 384
        _S33 = (_S34 & 2U) == 0U;

#line 384
    }
    else
    {

#line 384
        _S33 = false;

#line 384
    }

#line 384
    uint len_1;

#line 384
    if(_S33)
    {

#line 384
        len_1 = 1U;


        for(;;)
        {

#line 387
            bool _S35 = len_1 <= 64U;

#line 387
            _S30 = _S35;

#line 387
            if(_S35)
            {

#line 387
                uint _S36 = edge_at_0(x_2 + len_1, y_2, &kernelContext_6);

#line 387
                _S33 = (_S36 & 2U) != 0U;

#line 387
            }
            else
            {

#line 387
                _S33 = false;

#line 387
            }

#line 387
            if(_S33)
            {
            }
            else
            {

#line 387
                break;
            }

#line 387
            len_1 = len_1 + 1U;

#line 387
        }



        if(_S30)
        {

#line 391
            int _S37 = horizontal_turn_0(x_2, y_2, &kernelContext_6);

#line 391
            int _S38 = horizontal_turn_0(x_2 + len_1, y_2, &kernelContext_6);

#line 391
            blend_line_0(index_0, 1U, len_1, width_1, _S37, _S38, &kernelContext_6);

#line 391
        }

#line 384
    }

#line 399
    if((own_0 & 1U) != 0U)
    {

#line 399
        uint _S39 = edge_at_0(x_2, y_2 - 1U, &kernelContext_6);

#line 399
        _S33 = (_S39 & 1U) == 0U;

#line 399
    }
    else
    {

#line 399
        _S33 = false;

#line 399
    }

#line 399
    if(_S33)
    {

#line 399
        len_1 = 1U;


        for(;;)
        {

#line 402
            bool _S40 = len_1 <= 64U;

#line 402
            _S31 = _S40;

#line 402
            if(_S40)
            {

#line 402
                uint _S41 = edge_at_0(x_2, y_2 + len_1, &kernelContext_6);

#line 402
                _S33 = (_S41 & 1U) != 0U;

#line 402
            }
            else
            {

#line 402
                _S33 = false;

#line 402
            }

#line 402
            if(_S33)
            {
            }
            else
            {

#line 402
                break;
            }

#line 402
            len_1 = len_1 + 1U;

#line 402
        }



        if(_S31)
        {

#line 406
            int _S42 = vertical_turn_0(x_2, y_2, &kernelContext_6);

#line 406
            int _S43 = vertical_turn_0(x_2, y_2 + len_1, &kernelContext_6);

#line 406
            blend_line_0(index_0, width_1, len_1, 1U, _S42, _S43, &kernelContext_6);

#line 406
        }

#line 399
    }

#line 411
    return;
}

