#include <metal_stdlib>
#include <metal_math>
#include <metal_texture>
using namespace metal;

#line 60 "shaders/cmaa2_shapes.slang"
struct Cmaa2Params_0
{
    uint viewport_x_0;
    uint viewport_y_0;
    uint candidate_capacity_0;
    uint item_capacity_0;
};


#line 355
struct KernelContext_0
{
    atomic<uint> device* control_0;
    Cmaa2Params_0 constant* params_0;
    uint device* candidates_0;
    uint device* edges_0;
    uint device* items_0;
    texture2d<float, access::sample> source_0;
    atomic<uint> device* accum_0;
};


#line 168
uint edge_at_0(uint x_0, uint y_0, KernelContext_0 thread* kernelContext_0)
{

#line 168
    bool _S1;

    if(x_0 >= (kernelContext_0->params_0->viewport_x_0))
    {

#line 170
        _S1 = true;

#line 170
    }
    else
    {

#line 170
        _S1 = y_0 >= (kernelContext_0->params_0->viewport_y_0);

#line 170
    }

#line 170
    if(_S1)
    {
        return 0U;
    }
    return *(kernelContext_0->edges_0+(y_0 * kernelContext_0->params_0->viewport_x_0 + x_0));
}


#line 168
uint edge_at_1(uint x_1, uint y_1, KernelContext_0 thread* kernelContext_1)
{

#line 168
    bool _S2;

    if(x_1 >= (kernelContext_1->params_0->viewport_x_0))
    {

#line 170
        _S2 = true;

#line 170
    }
    else
    {

#line 170
        _S2 = y_1 >= (kernelContext_1->params_0->viewport_y_0);

#line 170
    }

#line 170
    if(_S2)
    {
        return 0U;
    }
    return *(kernelContext_1->edges_0+(y_1 * kernelContext_1->params_0->viewport_x_0 + x_1));
}


#line 185
int horizontal_turn_0(uint column_0, uint row_0, KernelContext_0 thread* kernelContext_2)
{

#line 185
    uint _S3 = edge_at_1(column_0, row_0, kernelContext_2);

    bool below_0 = (_S3 & 1U) != 0U;

#line 187
    bool above_0;
    if(row_0 > 0U)
    {

#line 188
        uint _S4 = edge_at_1(column_0, row_0 - 1U, kernelContext_2);

#line 188
        above_0 = (_S4 & 1U) != 0U;

#line 188
    }
    else
    {

#line 188
        above_0 = false;

#line 188
    }

#line 188
    bool _S5;
    if(above_0)
    {

#line 189
        _S5 = !below_0;

#line 189
    }
    else
    {

#line 189
        _S5 = false;

#line 189
    }

#line 189
    if(_S5)
    {
        return int(1);
    }
    if(below_0)
    {

#line 193
        above_0 = !above_0;

#line 193
    }
    else
    {

#line 193
        above_0 = false;

#line 193
    }

#line 193
    if(above_0)
    {
        return int(-1);
    }
    return int(0);
}


#line 229
float boundary_height_0(float t_0, int start_0, int end_0, bool u_shape_0)
{
    if(u_shape_0)
    {
        return -0.5f * float(start_0) * (2.0f * abs(t_0 - 0.5f)) * 0.5f;
    }
    return -0.5f * float(start_0) * (1.0f - t_0) - 0.5f * float(end_0) * t_0;
}


#line 249
void append_item_0(uint target_0, uint from_0, float share_0, KernelContext_0 thread* kernelContext_3)
{
    if(share_0 < 0.001953125f)
    {
        return;
    }
    uint slot_0 = atomic_fetch_add_explicit(kernelContext_3->control_0+int(1), 1U, memory_order_relaxed);
    if(slot_0 < (kernelContext_3->params_0->item_capacity_0))
    {
        uint _S6 = slot_0 * 3U;

#line 258
        *(kernelContext_3->items_0+_S6) = target_0;
        *(kernelContext_3->items_0+(_S6 + 1U)) = from_0;
        *(kernelContext_3->items_0+(_S6 + 2U)) = uint(min(share_0, 0.5f) * 1.048576e+06f);

#line 256
    }

#line 262
    return;
}


#line 286
void emit_line_0(uint first_0, uint stride_0, uint len_0, uint offset_0, int start_1, int end_1, KernelContext_0 thread* kernelContext_4)
{

#line 286
    bool _S7;

    if(start_1 == int(0))
    {

#line 288
        _S7 = true;

#line 288
    }
    else
    {

#line 288
        _S7 = end_1 == int(0);

#line 288
    }

#line 288
    if(_S7)
    {
        return;
    }
    float _S8 = 1.0f / float(len_0);
    bool _S9 = start_1 == end_1;

#line 293
    uint i_0 = 0U;
    for(;;)
    {

#line 294
        if(i_0 < len_0)
        {
        }
        else
        {

#line 294
            break;
        }

#line 301
        float near_0 = boundary_height_0(float(i_0) * _S8, start_1, end_1, _S9);
        uint _S10 = i_0 + 1U;

#line 302
        float far_0 = boundary_height_0(float(_S10) * _S8, start_1, end_1, _S9);

#line 309
        if(near_0 <= 0.0f)
        {

#line 309
            _S7 = far_0 <= 0.0f;

#line 309
        }
        else
        {

#line 309
            _S7 = false;

#line 309
        }

#line 309
        float below_1;

#line 309
        float above_1;

#line 309
        if(_S7)
        {

#line 309
            below_1 = -0.5f * (near_0 + far_0);

#line 309
            above_1 = 0.0f;

#line 309
        }
        else
        {

#line 309
            bool _S11;

#line 314
            if(near_0 >= 0.0f)
            {

#line 314
                _S11 = far_0 >= 0.0f;

#line 314
            }
            else
            {

#line 314
                _S11 = false;

#line 314
            }

#line 314
            if(_S11)
            {

                float _S12 = 0.5f * (near_0 + far_0);

#line 317
                below_1 = 0.0f;

#line 317
                above_1 = _S12;

#line 314
            }
            else
            {

#line 321
                float crossing_0 = near_0 / (near_0 - far_0);
                float _S13 = 0.5f * abs(min(near_0, far_0));

#line 322
                if(near_0 < 0.0f)
                {

#line 322
                    below_1 = crossing_0;

#line 322
                }
                else
                {

#line 322
                    below_1 = 1.0f - crossing_0;

#line 322
                }

#line 322
                float _S14 = _S13 * below_1;
                float _S15 = 0.5f * max(near_0, far_0);

#line 323
                if(near_0 > 0.0f)
                {

#line 323
                    above_1 = crossing_0;

#line 323
                }
                else
                {

#line 323
                    above_1 = 1.0f - crossing_0;

#line 323
                }

#line 323
                float _S16 = _S15 * above_1;

#line 323
                below_1 = _S14;

#line 323
                above_1 = _S16;

#line 314
            }

#line 309
        }

#line 326
        uint pixel_0 = first_0 + i_0 * stride_0;


        uint _S17 = pixel_0 - offset_0;

#line 329
        append_item_0(pixel_0, _S17, below_1, kernelContext_4);

#line 329
        append_item_0(_S17, pixel_0, above_1, kernelContext_4);

#line 294
        i_0 = _S10;

#line 294
    }

#line 332
    return;
}


#line 203
int vertical_turn_0(uint column_1, uint row_1, KernelContext_0 thread* kernelContext_5)
{

#line 203
    uint _S18 = edge_at_1(column_1, row_1, kernelContext_5);

    bool right_0 = (_S18 & 2U) != 0U;

#line 205
    bool left_0;
    if(column_1 > 0U)
    {

#line 206
        uint _S19 = edge_at_1(column_1 - 1U, row_1, kernelContext_5);

#line 206
        left_0 = (_S19 & 2U) != 0U;

#line 206
    }
    else
    {

#line 206
        left_0 = false;

#line 206
    }

#line 206
    bool _S20;
    if(left_0)
    {

#line 207
        _S20 = !right_0;

#line 207
    }
    else
    {

#line 207
        _S20 = false;

#line 207
    }

#line 207
    if(_S20)
    {
        return int(1);
    }
    if(right_0)
    {

#line 211
        left_0 = !left_0;

#line 211
    }
    else
    {

#line 211
        left_0 = false;

#line 211
    }

#line 211
    if(left_0)
    {
        return int(-1);
    }
    return int(0);
}


#line 343
[[kernel]] void shapesMain(uint3 thread_0 [[thread_position_in_grid]], atomic<uint> device* control_1 [[buffer(1)]], Cmaa2Params_0 constant* params_1 [[buffer(0)]], uint device* candidates_1 [[buffer(3)]], uint device* edges_1 [[buffer(2)]], uint device* items_1 [[buffer(5)]], texture2d<float, access::sample> source_1 [[texture(0)]], atomic<uint> device* accum_1 [[buffer(4)]])
{

#line 343
    bool _S21;

#line 343
    bool _S22;

#line 343
    thread KernelContext_0 kernelContext_6;

#line 343
    (&kernelContext_6)->control_0 = control_1;

#line 343
    (&kernelContext_6)->params_0 = params_1;

#line 343
    (&kernelContext_6)->candidates_0 = candidates_1;

#line 343
    (&kernelContext_6)->edges_0 = edges_1;

#line 343
    (&kernelContext_6)->items_0 = items_1;

#line 343
    (&kernelContext_6)->source_0 = source_1;

#line 343
    (&kernelContext_6)->accum_0 = accum_1;

#line 348
    uint _S23 = atomic_load_explicit(control_1+int(0), memory_order_relaxed);
    uint slot_1 = thread_0.x;
    if(slot_1 >= (min(_S23, (&kernelContext_6)->params_0->candidate_capacity_0)))
    {
        return;
    }

    uint device* _S24 = (&kernelContext_6)->candidates_0+slot_1;

#line 355
    uint index_0 = *_S24;
    uint width_0 = (&kernelContext_6)->params_0->viewport_x_0;
    uint x_2 = *_S24 % (&kernelContext_6)->params_0->viewport_x_0;
    uint y_2 = index_0 / width_0;
    uint device* _S25 = (&kernelContext_6)->edges_0+index_0;

#line 359
    uint own_0 = *_S25;

#line 359
    bool _S26;



    if(((*_S25) & 2U) != 0U)
    {

#line 363
        uint _S27 = edge_at_0(x_2 - 1U, y_2, &kernelContext_6);

#line 363
        _S26 = (_S27 & 2U) == 0U;

#line 363
    }
    else
    {

#line 363
        _S26 = false;

#line 363
    }

#line 363
    uint len_1;

#line 363
    if(_S26)
    {

#line 363
        len_1 = 1U;


        for(;;)
        {

#line 366
            bool _S28 = len_1 <= 64U;

#line 366
            _S21 = _S28;

#line 366
            if(_S28)
            {

#line 366
                uint _S29 = edge_at_0(x_2 + len_1, y_2, &kernelContext_6);

#line 366
                _S26 = (_S29 & 2U) != 0U;

#line 366
            }
            else
            {

#line 366
                _S26 = false;

#line 366
            }

#line 366
            if(_S26)
            {
            }
            else
            {

#line 366
                break;
            }

#line 366
            len_1 = len_1 + 1U;

#line 366
        }



        if(_S21)
        {

#line 370
            int _S30 = horizontal_turn_0(x_2, y_2, &kernelContext_6);

#line 370
            int _S31 = horizontal_turn_0(x_2 + len_1, y_2, &kernelContext_6);

#line 370
            emit_line_0(index_0, 1U, len_1, width_0, _S30, _S31, &kernelContext_6);

#line 370
        }

#line 363
    }

#line 378
    if((own_0 & 1U) != 0U)
    {

#line 378
        uint _S32 = edge_at_0(x_2, y_2 - 1U, &kernelContext_6);

#line 378
        _S26 = (_S32 & 1U) == 0U;

#line 378
    }
    else
    {

#line 378
        _S26 = false;

#line 378
    }

#line 378
    if(_S26)
    {

#line 378
        len_1 = 1U;


        for(;;)
        {

#line 381
            bool _S33 = len_1 <= 64U;

#line 381
            _S22 = _S33;

#line 381
            if(_S33)
            {

#line 381
                uint _S34 = edge_at_0(x_2, y_2 + len_1, &kernelContext_6);

#line 381
                _S26 = (_S34 & 1U) != 0U;

#line 381
            }
            else
            {

#line 381
                _S26 = false;

#line 381
            }

#line 381
            if(_S26)
            {
            }
            else
            {

#line 381
                break;
            }

#line 381
            len_1 = len_1 + 1U;

#line 381
        }



        if(_S22)
        {

#line 385
            int _S35 = vertical_turn_0(x_2, y_2, &kernelContext_6);

#line 385
            int _S36 = vertical_turn_0(x_2, y_2 + len_1, &kernelContext_6);

#line 385
            emit_line_0(index_0, width_0, len_1, 1U, _S35, _S36, &kernelContext_6);

#line 385
        }

#line 378
    }

#line 390
    return;
}


#line 401
[[kernel]] void accumulateMain(uint3 thread_1 [[thread_position_in_grid]], atomic<uint> device* control_2 [[buffer(1)]], Cmaa2Params_0 constant* params_2 [[buffer(0)]], uint device* candidates_2 [[buffer(3)]], uint device* edges_2 [[buffer(2)]], uint device* items_2 [[buffer(5)]], texture2d<float, access::sample> source_2 [[texture(0)]], atomic<uint> device* accum_2 [[buffer(4)]])
{

#line 401
    thread KernelContext_0 kernelContext_7;

#line 401
    (&kernelContext_7)->control_0 = control_2;

#line 401
    (&kernelContext_7)->params_0 = params_2;

#line 401
    (&kernelContext_7)->candidates_0 = candidates_2;

#line 401
    (&kernelContext_7)->edges_0 = edges_2;

#line 401
    (&kernelContext_7)->items_0 = items_2;

#line 401
    (&kernelContext_7)->source_0 = source_2;

#line 401
    (&kernelContext_7)->accum_0 = accum_2;

    uint _S37 = atomic_load_explicit(control_2+int(1), memory_order_relaxed);
    uint slot_2 = thread_1.x;
    if(slot_2 >= (min(_S37, (&kernelContext_7)->params_0->item_capacity_0)))
    {
        return;
    }

    uint _S38 = slot_2 * 3U;

#line 410
    uint device* _S39 = (&kernelContext_7)->items_0+_S38;

#line 410
    uint target_1 = *_S39;
    uint from_1 = *((&kernelContext_7)->items_0+(_S38 + 1U));
    uint weight_0 = *((&kernelContext_7)->items_0+(_S38 + 2U));
    uint pixels_0 = (&kernelContext_7)->params_0->viewport_x_0 * (&kernelContext_7)->params_0->viewport_y_0;

#line 413
    bool _S40;
    if((*_S39) >= pixels_0)
    {

#line 414
        _S40 = true;

#line 414
    }
    else
    {

#line 414
        _S40 = from_1 >= pixels_0;

#line 414
    }

#line 414
    if(_S40)
    {
        return;
    }

    uint width_1 = (&kernelContext_7)->params_0->viewport_x_0;
    uint _S41 = from_1 % (&kernelContext_7)->params_0->viewport_x_0;

#line 420
    int _S42 = int(_S41);

#line 420
    uint _S43 = from_1 / width_1;



    int3 _S44 = int3(int2(_S42, int(_S43)), int(0));

#line 424
    float3 color_0 = saturate((((&kernelContext_7)->source_0).read(vec<uint,2>(((_S44)).xy), uint(((_S44)).z))).xyz);
    float share_1 = float(weight_0);

    uint _S45 = target_1 * 4U;

#line 427
    uint _S46 = atomic_fetch_add_explicit((&kernelContext_7)->accum_0+_S45, uint(color_0.x * share_1), memory_order_relaxed);
    uint _S47 = atomic_fetch_add_explicit((&kernelContext_7)->accum_0+(_S45 + 1U), uint(color_0.y * share_1), memory_order_relaxed);
    uint _S48 = atomic_fetch_add_explicit((&kernelContext_7)->accum_0+(_S45 + 2U), uint(color_0.z * share_1), memory_order_relaxed);
    uint _S49 = atomic_fetch_add_explicit((&kernelContext_7)->accum_0+(_S45 + 3U), weight_0, memory_order_relaxed);
    return;
}

