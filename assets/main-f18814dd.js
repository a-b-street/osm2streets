import{S as re,i as ie,s as ne,e as De,a as g,t as v,g as Ie,b as C,c as je,d as h,f as Nt,h as Bt,m as zt,j as Z,o as vt,k as Te,l as Pt,n as Jt,p as c,q as wt,u as Ct,r as Ot,v as Lt,w as Rt,x as ve,L as At,y as Ee,z as B,A as z,B as He,C as P,D as Dt,E as Me,F as R,G as Se,H as Mt,I as f,J as Ae,K as ue,M as V,N as tt,O as Tt,P as he,Q as Ge,R as y,T as K,U as nt,V as St,W as lt,X as me,Y as Ne,Z as Et,_ as Fe,$ as It,a0 as We,a1 as ot,a2 as qe,a3 as Ze,a4 as Ue,a5 as Qe,a6 as jt,a7 as Ht,a8 as Gt,a9 as Ft,aa as Wt,ab as qt,ac as Ut,ad as Vt,ae as Kt,af as Zt,ag as Qt,ah as Xt,ai as Yt,aj as $t,ak as xt,al as en}from"./RenderLanePolygons-e25f4cfc.js";const tn=s=>({features:s[0]&16,data:s[0]&16,map:s[0]&4,close:s[0]&1}),st=s=>{var e;return{features:s[4],data:(e=s[4])==null?void 0:e[0],map:s[2],close:s[31]}};function rt(s){let e,n,t=(s[4]||s[3]instanceof Te.Marker)&&it(s);return{c(){e=c("div"),t&&t.c()},m(l,o){g(l,e,o),t&&t.m(e,null),s[32](e),n=!0},p(l,o){l[4]||l[3]instanceof Te.Marker?t?(t.p(l,o),o[0]&24&&v(t,1)):(t=it(l),t.c(),v(t,1),t.m(e,null)):t&&(Ie(),C(t,1,1,()=>{t=null}),je())},i(l){n||(v(t),n=!0)},o(l){C(t),n=!1},d(l){l&&h(e),t&&t.d(),s[32](null)}}}function it(s){let e;const n=s[30].default,t=wt(n,s,s[29],st);return{c(){t&&t.c()},m(l,o){t&&t.m(l,o),e=!0},p(l,o){t&&t.p&&(!e||o[0]&536870933)&&Ct(t,n,l,l[29],e?Lt(n,l[29],o,tn):Ot(l[29]),st)},i(l){e||(v(t,l),e=!0)},o(l){C(t,l),e=!1},d(l){t&&t.d(l)}}}function nn(s){let e,n,t=s[9].default&&rt(s);return{c(){t&&t.c(),e=De()},m(l,o){t&&t.m(l,o),g(l,e,o),n=!0},p(l,o){l[9].default?t?(t.p(l,o),o[0]&512&&v(t,1)):(t=rt(l),t.c(),v(t,1),t.m(e.parentNode,e)):t&&(Ie(),C(t,1,1,()=>{t=null}),je())},i(l){n||(v(t),n=!0)},o(l){C(t),n=!1},d(l){l&&h(e),t&&t.d(l)}}}function ln(s,e,n){let t,l,o,r,u,i,{$$slots:a={},$$scope:b}=e;const O=Nt(a);let{closeButton:T=void 0}=e,{closeOnClickOutside:w=!0}=e,{closeOnClickInside:I=!1}=e,{closeOnMove:p=!1}=e,{openOn:k="click"}=e,{openIfTopMost:j=!0}=e,{focusAfterOpen:S=!0}=e,{anchor:G=void 0}=e,{offset:F=void 0}=e,{popupClass:Y=void 0}=e,{maxWidth:D=void 0}=e,{lngLat:W=void 0}=e,{html:U=void 0}=e,{open:E=!1}=e;const $=Bt(),{map:Q,popupTarget:_,layerEvent:le,layer:q,eventTopMost:oe}=zt();Z(s,Q,m=>n(2,o=m)),Z(s,_,m=>n(3,u=m)),Z(s,le,m=>n(28,r=m)),Z(s,q,m=>n(35,i=m));const H=["click","dblclick","contextmenu"];let d,N=!1,J;function L(){if(!d)return;let m=d.getElement();!m||m===J||(J=m,k==="hover"&&(J.style.pointerEvents="none"),J.addEventListener("mouseenter",()=>{n(24,N=!0)},{passive:!0}),J.addEventListener("mouseleave",()=>{n(24,N=!1)},{passive:!0}),J.addEventListener("click",()=>{I&&n(0,E=!1)},{passive:!0}))}vt(()=>{if(o)return o.on("click",ae),o.on("contextmenu",ae),typeof u=="string"&&(o.on("click",u,fe),o.on("dblclick",u,fe),o.on("contextmenu",u,fe),o.on("mousemove",u,ge),o.on("mouseleave",u,Ce),o.on("touchstart",u,we),o.on("touchend",u,be)),()=>{o!=null&&o.loaded()&&(d==null||d.remove(),o.off("click",ae),o.off("contextmenu",ae),u instanceof Te.Marker?u.getPopup()===d&&u.setPopup(void 0):typeof u=="string"&&(o.off("click",u,fe),o.off("dblclick",u,fe),o.off("contextmenu",u,fe),o.off("mousemove",u,ge),o.off("mouseleave",u,Ce),o.off("touchstart",u,we),o.off("touchend",u,be)))}});function se(m){return j?!("marker"in m)&&!Rt(m)&&oe(m)!==i:!1}let x=null,ee="normal";function fe(m){m.type===k&&(se(m)||("layerType"in m?m.layerType==="deckgl"?(n(10,W=m.coordinate),n(4,x=m.object?[m.object]:null)):(n(10,W=m.lngLat),n(4,x=m.features??[])):(n(10,W=m.lngLat),n(4,x=m.features??[])),setTimeout(()=>n(0,E=!0))))}let pe=null;function we(m){pe=m.point}function be(m){if(!pe||k!=="hover")return;let de=pe.dist(m.point);pe=null,de<3&&(n(10,W=m.lngLat),n(4,x=m.features??[]),d.isOpen()?n(25,ee="justOpened"):(n(25,ee="opening"),n(0,E=!0)))}function Ce(m){k!=="hover"||pe||ee!=="normal"||(n(0,E=!1),n(4,x=null))}function ge(m){if(!(k!=="hover"||pe||ee!=="normal")){if(se(m)){n(0,E=!1),n(4,x=null);return}n(0,E=!0),n(4,x=m.features??[]),n(10,W=m.lngLat)}}function ae(m){if(ee==="justOpened"){n(25,ee="normal");return}if(!w)return;let de=[J,u instanceof Te.Marker?u==null?void 0:u.getElement():null];E&&d.isOpen()&&!de.some(_e=>_e==null?void 0:_e.contains(m.originalEvent.target))&&(m.type==="contextmenu"&&k==="contextmenu"||m.type!=="contextmenu")&&n(0,E=!1)}Pt(()=>{o&&(d!=null&&d.isOpen())&&d.remove()});let ce;const Re=()=>n(0,E=!1);function ke(m){ve[m?"unshift":"push"](()=>{ce=m,n(1,ce)})}return s.$$set=m=>{"closeButton"in m&&n(11,T=m.closeButton),"closeOnClickOutside"in m&&n(12,w=m.closeOnClickOutside),"closeOnClickInside"in m&&n(13,I=m.closeOnClickInside),"closeOnMove"in m&&n(14,p=m.closeOnMove),"openOn"in m&&n(15,k=m.openOn),"openIfTopMost"in m&&n(16,j=m.openIfTopMost),"focusAfterOpen"in m&&n(17,S=m.focusAfterOpen),"anchor"in m&&n(18,G=m.anchor),"offset"in m&&n(19,F=m.offset),"popupClass"in m&&n(20,Y=m.popupClass),"maxWidth"in m&&n(21,D=m.maxWidth),"lngLat"in m&&n(10,W=m.lngLat),"html"in m&&n(22,U=m.html),"open"in m&&n(0,E=m.open),"$$scope"in m&&n(29,b=m.$$scope)},s.$$.update=()=>{if(s.$$.dirty[0]&14336&&n(27,t=T??(!w&&!I)),s.$$.dirty[0]&146685952&&(d||(n(23,d=new Te.Popup({closeButton:t,closeOnClick:!1,closeOnMove:p,focusAfterOpen:S,maxWidth:D,className:Y,anchor:G,offset:F})),J=d.getElement(),d.on("open",()=>{n(0,E=!0),L(),$("open",d)}),d.on("close",()=>{n(0,E=!1),$("close",d)}),d.on("hover",()=>{$("hover",d)}))),s.$$.dirty[0]&8421384&&d&&u instanceof Te.Marker&&(k==="click"?u.setPopup(d):u.getPopup()===d&&u.setPopup(void 0)),s.$$.dirty[0]&268468224&&H.includes(k)&&(r==null?void 0:r.type)===k&&(fe(r),Jt(le,r=null,r)),s.$$.dirty[0]&268468224&&n(26,l=k==="hover"&&((r==null?void 0:r.type)==="mousemove"||(r==null?void 0:r.type)==="mouseenter")),s.$$.dirty[0]&352354304&&k==="hover"&&le&&(l&&r&&(r.layerType==="deckgl"?(n(10,W=r.coordinate),n(4,x=r.object?[r.object]:null)):(n(10,W=r.lngLat),n(4,x=r.features??[]))),n(0,E=(l||N)??!1)),s.$$.dirty[0]&12582914&&(ce?d.setDOMContent(ce):U&&d.setHTML(U)),s.$$.dirty[0]&8389632&&W&&d.setLngLat(W),s.$$.dirty[0]&41943045&&o){let m=d.isOpen();E&&!m?(d.addTo(o),ee==="opening"&&n(25,ee="justOpened")):!E&&m&&d.remove()}},[E,ce,o,u,x,Q,_,le,q,O,W,T,w,I,p,k,j,S,G,F,Y,D,U,d,N,ee,l,t,r,b,a,Re,ke]}class Xe extends re{constructor(e){super(),ie(this,e,ln,nn,ne,{closeButton:11,closeOnClickOutside:12,closeOnClickInside:13,closeOnMove:14,openOn:15,openIfTopMost:16,focusAfterOpen:17,anchor:18,offset:19,popupClass:20,maxWidth:21,lngLat:10,html:22,open:0},null,[-1,-1])}}function on(s){let e;const n=s[16].default,t=wt(n,s,s[24],null);return{c(){t&&t.c()},m(l,o){t&&t.m(l,o),e=!0},p(l,o){t&&t.p&&(!e||o&16777216)&&Ct(t,n,l,l[24],e?Lt(n,l[24],o,null):Ot(l[24]),null)},i(l){e||(v(t,l),e=!0)},o(l){C(t,l),e=!1},d(l){t&&t.d(l)}}}function sn(s){let e,n,t;function l(r){s[17](r)}let o={id:s[1],type:"symbol",source:s[2],sourceLayer:s[3],beforeId:s[4],beforeLayerType:s[5],paint:s[6],layout:s[7],filter:s[8],applyToClusters:s[9],minzoom:s[10],maxzoom:s[11],hoverCursor:s[12],manageHoverState:s[13],eventsIfTopMost:s[14],interactive:s[15],$$slots:{default:[on]},$$scope:{ctx:s}};return s[0]!==void 0&&(o.hovered=s[0]),e=new At({props:o}),ve.push(()=>Ee(e,"hovered",l)),e.$on("click",s[18]),e.$on("dblclick",s[19]),e.$on("contextmenu",s[20]),e.$on("mouseenter",s[21]),e.$on("mousemove",s[22]),e.$on("mouseleave",s[23]),{c(){B(e.$$.fragment)},m(r,u){z(e,r,u),t=!0},p(r,[u]){const i={};u&2&&(i.id=r[1]),u&4&&(i.source=r[2]),u&8&&(i.sourceLayer=r[3]),u&16&&(i.beforeId=r[4]),u&32&&(i.beforeLayerType=r[5]),u&64&&(i.paint=r[6]),u&128&&(i.layout=r[7]),u&256&&(i.filter=r[8]),u&512&&(i.applyToClusters=r[9]),u&1024&&(i.minzoom=r[10]),u&2048&&(i.maxzoom=r[11]),u&4096&&(i.hoverCursor=r[12]),u&8192&&(i.manageHoverState=r[13]),u&16384&&(i.eventsIfTopMost=r[14]),u&32768&&(i.interactive=r[15]),u&16777216&&(i.$$scope={dirty:u,ctx:r}),!n&&u&1&&(n=!0,i.hovered=r[0],He(()=>n=!1)),e.$set(i)},i(r){t||(v(e.$$.fragment,r),t=!0)},o(r){C(e.$$.fragment,r),t=!1},d(r){P(e,r)}}}function rn(s,e,n){let{$$slots:t={},$$scope:l}=e,{id:o=Dt("symbol")}=e,{source:r=void 0}=e,{sourceLayer:u=void 0}=e,{beforeId:i=void 0}=e,{beforeLayerType:a=void 0}=e,{paint:b=void 0}=e,{layout:O=void 0}=e,{filter:T=void 0}=e,{applyToClusters:w=void 0}=e,{minzoom:I=void 0}=e,{maxzoom:p=void 0}=e,{hoverCursor:k=void 0}=e,{manageHoverState:j=!1}=e,{hovered:S=null}=e,{eventsIfTopMost:G=!1}=e,{interactive:F=!0}=e;function Y(_){S=_,n(0,S)}function D(_){Me.call(this,s,_)}function W(_){Me.call(this,s,_)}function U(_){Me.call(this,s,_)}function E(_){Me.call(this,s,_)}function $(_){Me.call(this,s,_)}function Q(_){Me.call(this,s,_)}return s.$$set=_=>{"id"in _&&n(1,o=_.id),"source"in _&&n(2,r=_.source),"sourceLayer"in _&&n(3,u=_.sourceLayer),"beforeId"in _&&n(4,i=_.beforeId),"beforeLayerType"in _&&n(5,a=_.beforeLayerType),"paint"in _&&n(6,b=_.paint),"layout"in _&&n(7,O=_.layout),"filter"in _&&n(8,T=_.filter),"applyToClusters"in _&&n(9,w=_.applyToClusters),"minzoom"in _&&n(10,I=_.minzoom),"maxzoom"in _&&n(11,p=_.maxzoom),"hoverCursor"in _&&n(12,k=_.hoverCursor),"manageHoverState"in _&&n(13,j=_.manageHoverState),"hovered"in _&&n(0,S=_.hovered),"eventsIfTopMost"in _&&n(14,G=_.eventsIfTopMost),"interactive"in _&&n(15,F=_.interactive),"$$scope"in _&&n(24,l=_.$$scope)},[S,o,r,u,i,a,b,O,T,w,I,p,k,j,G,F,t,Y,D,W,U,E,$,Q,l]}class un extends re{constructor(e){super(),ie(this,e,rn,sn,ne,{id:1,source:2,sourceLayer:3,beforeId:4,beforeLayerType:5,paint:6,layout:7,filter:8,applyToClusters:9,minzoom:10,maxzoom:11,hoverCursor:12,manageHoverState:13,hovered:0,eventsIfTopMost:14,interactive:15})}}function fn(s){let e,n,t,l,o,r,u,i,a,b;return{c(){e=c("div"),n=c("label"),t=R(`Basemap:
    `),l=c("select"),o=c("option"),o.textContent="MapTiler Dataviz",r=c("option"),r.textContent="MapTiler Streets",u=c("option"),u.textContent="MapTiler Satellite",i=c("option"),i.textContent="Blank",o.__value="dataviz",Se(o,o.__value),r.__value="streets",Se(r,r.__value),u.__value="hybrid",Se(u,u.__value),i.__value="blank",Se(i,i.__value),s[0]===void 0&&Mt(()=>s[1].call(l))},m(O,T){g(O,e,T),f(e,n),f(n,t),f(n,l),f(l,o),f(l,r),f(l,u),f(l,i),Ae(l,s[0],!0),a||(b=ue(l,"change",s[1]),a=!0)},p(O,[T]){T&1&&Ae(l,O[0])},i:V,o:V,d(O){O&&h(e),a=!1,b()}}}function an(s,e,n){let t;Z(s,tt,o=>n(0,t=o));function l(){t=Tt(this),tt.set(t)}return[t,l]}class cn extends re{constructor(e){super(),ie(this,e,an,fn,ne,{})}}function ut(s,e,n){const t=s.slice();return t[1]=e[n][0],t[2]=e[n][1],t}function ft(s){let e,n,t,l=s[1]+"",o,r;return{c(){e=c("li"),n=c("span"),t=y(),o=R(l),r=y(),K(n,"class","svelte-kzgqtg"),nt(n,"background",s[2])},m(u,i){g(u,e,i),f(e,n),f(e,t),f(e,o),f(e,r)},p(u,i){i&1&&nt(n,"background",u[2]),i&1&&l!==(l=u[1]+"")&&St(o,l)},d(u){u&&h(e)}}}function mn(s){let e,n=he(s[0]),t=[];for(let l=0;l<n.length;l+=1)t[l]=ft(ut(s,n,l));return{c(){e=c("ul");for(let l=0;l<t.length;l+=1)t[l].c()},m(l,o){g(l,e,o);for(let r=0;r<t.length;r+=1)t[r]&&t[r].m(e,null)},p(l,[o]){if(o&1){n=he(l[0]);let r;for(r=0;r<n.length;r+=1){const u=ut(l,n,r);t[r]?t[r].p(u,o):(t[r]=ft(u),t[r].c(),t[r].m(e,null))}for(;r<t.length;r+=1)t[r].d(1);t.length=n.length}},i:V,o:V,d(l){l&&h(e),Ge(t,l)}}}function pn(s,e,n){let{rows:t}=e;return s.$$set=l=>{"rows"in l&&n(0,t=l.rows)},[t]}class dn extends re{constructor(e){super(),ie(this,e,pn,mn,ne,{rows:0})}}function _n(s){let e,n,t,l,o,r,u,i;return{c(){e=c("div"),n=c("label"),t=R(`Theme:
    `),l=c("select"),o=c("option"),o.textContent="Debug",r=c("option"),r.textContent="Realistic",o.__value="debug",Se(o,o.__value),r.__value="realistic",Se(r,r.__value),s[0]===void 0&&Mt(()=>s[1].call(l))},m(a,b){g(a,e,b),f(e,n),f(n,t),f(n,l),f(l,o),f(l,r),Ae(l,s[0],!0),u||(i=ue(l,"change",s[1]),u=!0)},p(a,[b]){b&1&&Ae(l,a[0])},i:V,o:V,d(a){a&&h(e),u=!1,i()}}}function gn(s,e,n){let t;Z(s,lt,o=>n(0,t=o));function l(){t=Tt(this),lt.set(t)}return[t,l]}class hn extends re{constructor(e){super(),ie(this,e,gn,_n,ne,{})}}const Je=Et(Ne());me.subscribe(s=>{Je.set(Ne())});function bn(s){let e,n=s[6].properties.kind+"",t;return{c(){e=c("p"),t=R(n)},m(l,o){g(l,e,o),f(e,t)},p(l,o){o&64&&n!==(n=l[6].properties.kind+"")&&St(t,n)},d(l){l&&h(e)}}}function kn(s){let e,n;return e=new Xe({props:{openOn:"hover",$$slots:{default:[bn,({data:t})=>({6:t}),({data:t})=>t?64:0]},$$scope:{ctx:s}}}),{c(){B(e.$$.fragment)},m(t,l){z(e,t,l),n=!0},p(t,l){const o={};l&192&&(o.$$scope={dirty:l,ctx:t}),e.$set(o)},i(t){n||(v(e.$$.fragment,t),n=!0)},o(t){C(e.$$.fragment,t),n=!1},d(t){P(e,t)}}}function yn(s){let e,n;const t=[We("block"),{paint:{"fill-color":ot("kind",s[4],"red"),"fill-opacity":.8}}];let l={$$slots:{default:[kn]},$$scope:{ctx:s}};for(let o=0;o<t.length;o+=1)l=qe(l,t[o]);return e=new Ze({props:l}),{c(){B(e.$$.fragment)},m(o,r){z(e,o,r),n=!0},p(o,r){const u=r&16?Ue(t,[t[0],{paint:{"fill-color":ot("kind",o[4],"red"),"fill-opacity":.8}}]):{};r&128&&(u.$$scope={dirty:r,ctx:o}),e.$set(u)},i(o){n||(v(e.$$.fragment,o),n=!0)},o(o){C(e.$$.fragment,o),n=!1},d(o){P(e,o)}}}function at(s){let e,n;return e=new dn({props:{rows:Object.entries(s[4])}}),{c(){B(e.$$.fragment)},m(t,l){z(e,t,l),n=!0},p:V,i(t){n||(v(e.$$.fragment,t),n=!0)},o(t){C(e.$$.fragment,t),n=!1},d(t){P(e,t)}}}function vn(s){let e,n,t,l,o,r,u,i,a,b,O,T,w,I;e=new Fe({props:{data:s[0],$$slots:{default:[yn]},$$scope:{ctx:s}}});let p=s[1]&&at(s);return{c(){B(e.$$.fragment),n=y(),t=c("div"),l=R(`Blocks
  `),o=c("button"),r=R("Clear"),i=y(),a=c("button"),a.textContent="Find all",b=y(),p&&p.c(),O=De(),o.disabled=u=!s[1]},m(k,j){z(e,k,j),g(k,n,j),g(k,t,j),f(t,l),f(t,o),f(o,r),f(t,i),f(t,a),g(k,b,j),p&&p.m(k,j),g(k,O,j),T=!0,w||(I=[ue(o,"click",s[2]),ue(a,"click",s[3])],w=!0)},p(k,[j]){const S={};j&1&&(S.data=k[0]),j&128&&(S.$$scope={dirty:j,ctx:k}),e.$set(S),(!T||j&2&&u!==(u=!k[1]))&&(o.disabled=u),k[1]?p?(p.p(k,j),j&2&&v(p,1)):(p=at(k),p.c(),v(p,1),p.m(O.parentNode,O)):p&&(Ie(),C(p,1,1,()=>{p=null}),je())},i(k){T||(v(e.$$.fragment,k),v(p),T=!0)},o(k){C(e.$$.fragment,k),C(p),T=!1},d(k){k&&(h(n),h(t),h(b),h(O)),P(e,k),p&&p.d(k),w=!1,It(I)}}}function wn(s,e,n){let t,l,o;Z(s,me,a=>n(5,l=a)),Z(s,Je,a=>n(0,o=a));function r(){Je.set(Ne())}function u(){Je.set(JSON.parse(l.findAllBlocks()))}let i={RoadAndSidewalk:"green",RoadAndCycleLane:"orange",CycleLaneAndSidewalk:"yellow",DualCarriageway:"purple",Unknown:"blue"};return s.$$.update=()=>{s.$$.dirty&1&&n(1,t=o.features.length>0)},[o,t,r,u,i]}class Cn extends re{constructor(e){super(),ie(this,e,wn,vn,ne,{})}}function On(s){let e,n;const t=[We("connected-roads"),{layout:{visibility:s[0]?"visible":"none"}},{paint:{"fill-color":"blue","fill-opacity":.5}}];let l={};for(let o=0;o<t.length;o+=1)l=qe(l,t[o]);return e=new Ze({props:l}),{c(){B(e.$$.fragment)},m(o,r){z(e,o,r),n=!0},p(o,r){const u=r&1?Ue(t,[t[0],{layout:{visibility:o[0]?"visible":"none"}},t[2]]):{};e.$set(u)},i(o){n||(v(e.$$.fragment,o),n=!0)},o(o){C(e.$$.fragment,o),n=!1},d(o){P(e,o)}}}function Ln(s){let e,n,t,l,o;e=new Fe({props:{data:s[1],$$slots:{default:[On]},$$scope:{ctx:s}}});function r(i){s[4](i)}let u={gj:s[1],name:"Roads connected to intersection",downloadable:!1};return s[0]!==void 0&&(u.show=s[0]),t=new Qe({props:u}),ve.push(()=>Ee(t,"show",r)),{c(){B(e.$$.fragment),n=y(),B(t.$$.fragment)},m(i,a){z(e,i,a),g(i,n,a),z(t,i,a),o=!0},p(i,[a]){const b={};a&2&&(b.data=i[1]),a&33&&(b.$$scope={dirty:a,ctx:i}),e.$set(b);const O={};a&2&&(O.gj=i[1]),!l&&a&1&&(l=!0,O.show=i[0],He(()=>l=!1)),t.$set(O)},i(i){o||(v(e.$$.fragment,i),v(t.$$.fragment,i),o=!0)},o(i){C(e.$$.fragment,i),C(t.$$.fragment,i),o=!1},d(i){i&&h(n),P(e,i),P(t,i)}}}function Mn(s,e,n){let t,l,o;Z(s,jt,i=>n(2,l=i)),Z(s,me,i=>n(3,o=i));let r=!0;function u(i){r=i,n(0,r)}return s.$$.update=()=>{s.$$.dirty&12&&n(1,t=o&&l?JSON.parse(o.debugRoadsConnectedToIntersectionGeojson(l.properties.id)):Ne())},[r,t,l,o,u]}class Tn extends re{constructor(e){super(),ie(this,e,Mn,Ln,ne,{})}}function Sn(s){let e,n;const t=[We("movements"),{layout:{visibility:s[0]?"visible":"none"}},{paint:{"fill-color":"blue","fill-opacity":.5}}];let l={};for(let o=0;o<t.length;o+=1)l=qe(l,t[o]);return e=new Ze({props:l}),{c(){B(e.$$.fragment)},m(o,r){z(e,o,r),n=!0},p(o,r){const u=r&1?Ue(t,[t[0],{layout:{visibility:o[0]?"visible":"none"}},t[2]]):{};e.$set(u)},i(o){n||(v(e.$$.fragment,o),n=!0)},o(o){C(e.$$.fragment,o),n=!1},d(o){P(e,o)}}}function In(s){let e,n,t,l,o;e=new Fe({props:{data:s[1],$$slots:{default:[Sn]},$$scope:{ctx:s}}});function r(i){s[4](i)}let u={gj:s[1],name:"Movement arrows",downloadable:!1};return s[0]!==void 0&&(u.show=s[0]),t=new Qe({props:u}),ve.push(()=>Ee(t,"show",r)),{c(){B(e.$$.fragment),n=y(),B(t.$$.fragment)},m(i,a){z(e,i,a),g(i,n,a),z(t,i,a),o=!0},p(i,[a]){const b={};a&2&&(b.data=i[1]),a&33&&(b.$$scope={dirty:a,ctx:i}),e.$set(b);const O={};a&2&&(O.gj=i[1]),!l&&a&1&&(l=!0,O.show=i[0],He(()=>l=!1)),t.$set(O)},i(i){o||(v(e.$$.fragment,i),v(t.$$.fragment,i),o=!0)},o(i){C(e.$$.fragment,i),C(t.$$.fragment,i),o=!1},d(i){i&&h(n),P(e,i),P(t,i)}}}function jn(s,e,n){let t,l,o;Z(s,Ht,i=>n(2,l=i)),Z(s,me,i=>n(3,o=i));let r=!0;function u(i){r=i,n(0,r)}return s.$$.update=()=>{s.$$.dirty&12&&n(1,t=o&&l?JSON.parse(o.debugMovementsFromLaneGeojson(l.properties.road,l.properties.index)):Ne())},[r,t,l,o,u]}class Nn extends re{constructor(e){super(),ie(this,e,jn,In,ne,{})}}function Bn(s){let e,n,t,l,o,r;return{c(){e=c("div"),n=c("label"),t=c("input"),l=R(`
    Clockwise ordering of roads`),K(t,"type","checkbox")},m(u,i){g(u,e,i),f(e,n),f(n,t),t.checked=s[0],f(n,l),o||(r=ue(t,"change",s[5]),o=!0)},p(u,[i]){i&1&&(t.checked=u[0])},i:V,o:V,d(u){u&&h(e),o=!1,r()}}}function zn(s,e,n){let t,l,o;Z(s,Gt,a=>n(2,t=a)),Z(s,jt,a=>n(3,l=a)),Z(s,me,a=>n(4,o=a));let r=[],u=!1;function i(){u=this.checked,n(0,u)}return s.$$.update=()=>{if(s.$$.dirty&31){for(let a of r)a.remove();if(n(1,r=[]),u&&l){let a=JSON.parse(o.debugClockwiseOrderingForIntersectionGeojson(l.properties.id));for(let b of a.features)r.push(new Ft.Popup({closeButton:!1,closeOnClick:!1,focusAfterOpen:!1}).setLngLat(b.geometry.coordinates).setHTML(b.properties.label).addTo(t))}}},[u,r,t,l,o,i]}class Pn extends re{constructor(e){super(),ie(this,e,zn,Bn,ne,{})}}function Jn(s){let e,n;const t=[We("debug-ids"),{layout:{"text-field":["get","id"],visibility:s[0]?"visible":"none"}},{paint:{"text-halo-color":["case",["==",["get","type"],"intersection"],"red","cyan"],"text-halo-width":3}}];let l={};for(let o=0;o<t.length;o+=1)l=qe(l,t[o]);return e=new un({props:l}),{c(){B(e.$$.fragment)},m(o,r){z(e,o,r),n=!0},p(o,r){const u=r&1?Ue(t,[t[0],{layout:{"text-field":["get","id"],visibility:o[0]?"visible":"none"}},t[2]]):{};e.$set(u)},i(o){n||(v(e.$$.fragment,o),n=!0)},o(o){C(e.$$.fragment,o),n=!1},d(o){P(e,o)}}}function Rn(s){let e,n,t,l,o;e=new Fe({props:{data:s[1],generateId:!0,$$slots:{default:[Jn]},$$scope:{ctx:s}}});function r(i){s[3](i)}let u={gj:s[1],name:"Debug IDs"};return s[0]!==void 0&&(u.show=s[0]),t=new Qe({props:u}),ve.push(()=>Ee(t,"show",r)),{c(){B(e.$$.fragment),n=y(),B(t.$$.fragment)},m(i,a){z(e,i,a),g(i,n,a),z(t,i,a),o=!0},p(i,[a]){const b={};a&2&&(b.data=i[1]),a&17&&(b.$$scope={dirty:a,ctx:i}),e.$set(b);const O={};a&2&&(O.gj=i[1]),!l&&a&1&&(l=!0,O.show=i[0],He(()=>l=!1)),t.$set(O)},i(i){o||(v(e.$$.fragment,i),v(t.$$.fragment,i),o=!0)},o(i){C(e.$$.fragment,i),C(t.$$.fragment,i),o=!1},d(i){i&&h(n),P(e,i),P(t,i)}}}function An(s,e,n){let t,l;Z(s,me,u=>n(2,l=u));let o=!1;function r(u){o=u,n(0,o)}return s.$$.update=()=>{s.$$.dirty&4&&n(1,t=l?JSON.parse(l.toGeojsonPlain()):Ne())},[o,t,l,r]}class Dn extends re{constructor(e){super(),ie(this,e,An,Rn,ne,{})}}function ct(s,e,n){const t=s.slice();return t[5]=e[n],t}function mt(s){const e=s.slice(),n=JSON.parse(e[0].crossing);return e[8]=n,e}function En(s){let e,n,t,l=s[8].kind+"",o,r,u=s[8].has_island&&Hn();return{c(){e=c("p"),n=c("u"),n.textContent="Crossing",t=R(": "),o=R(l),r=y(),u&&u.c()},m(i,a){g(i,e,a),f(e,n),f(e,t),f(e,o),f(e,r),u&&u.m(e,null)},p:V,d(i){i&&h(e),u&&u.d()}}}function Hn(s){let e;return{c(){e=R("(with an island)")},m(n,t){g(n,e,t)},d(n){n&&h(e)}}}function pt(s){let e,n=s[5]+"",t,l;return{c(){e=c("a"),t=R(n),l=R(","),K(e,"href","https://www.openstreetmap.org/node/"+s[5]),K(e,"target","_blank")},m(o,r){g(o,e,r),f(e,t),g(o,l,r)},p:V,d(o){o&&(h(e),h(l))}}}function Gn(s){let e,n,t,l,o,r=s[0].intersection_kind+"",u,i,a,b,O,T=s[0].control+"",w,I,p,k,j,S=s[0].movements+"",G,F,Y,D,W,U,E,$,Q,_,le,q=s[0].crossing&&En(mt(s)),oe=he(JSON.parse(s[0].osm_node_ids)),H=[];for(let d=0;d<oe.length;d+=1)H[d]=pt(ct(s,oe,d));return{c(){e=c("h2"),e.textContent=`Intersection #${s[0].id}`,n=y(),t=c("p"),l=c("u"),l.textContent="Kind",o=R(": "),u=R(r),i=y(),a=c("p"),b=c("u"),b.textContent="Control",O=R(": "),w=R(T),I=y(),p=c("p"),k=c("u"),k.textContent="Movements",j=R(": "),G=R(S),F=y(),q&&q.c(),Y=y(),D=c("p"),W=c("u"),W.textContent="OSM nodes",U=R(`:
  `);for(let d=0;d<H.length;d+=1)H[d].c();E=y(),$=c("div"),Q=c("button"),Q.textContent="Collapse intersection",K(Q,"type","button")},m(d,N){g(d,e,N),g(d,n,N),g(d,t,N),f(t,l),f(t,o),f(t,u),g(d,i,N),g(d,a,N),f(a,b),f(a,O),f(a,w),g(d,I,N),g(d,p,N),f(p,k),f(p,j),f(p,G),g(d,F,N),q&&q.m(d,N),g(d,Y,N),g(d,D,N),f(D,W),f(D,U);for(let J=0;J<H.length;J+=1)H[J]&&H[J].m(D,null);g(d,E,N),g(d,$,N),f($,Q),_||(le=ue(Q,"click",s[1]),_=!0)},p(d,[N]){if(d[0].crossing&&q.p(mt(d),N),N&1){oe=he(JSON.parse(d[0].osm_node_ids));let J;for(J=0;J<oe.length;J+=1){const L=ct(d,oe,J);H[J]?H[J].p(L,N):(H[J]=pt(L),H[J].c(),H[J].m(D,null))}for(;J<H.length;J+=1)H[J].d(1);H.length=oe.length}},i:V,o:V,d(d){d&&(h(e),h(n),h(t),h(i),h(a),h(I),h(p),h(F),h(Y),h(D),h(E),h($)),q&&q.d(d),Ge(H,d),_=!1,le()}}}function Fn(s,e,n){let t;Z(s,me,i=>n(4,t=i));let{data:l}=e,{close:o}=e,r=l.properties;function u(){t.collapseIntersection(r.id),me.set(t),o()}return s.$$set=i=>{"data"in i&&n(2,l=i.data),"close"in i&&n(3,o=i.close)},[r,u,l,o]}class Wn extends re{constructor(e){super(),ie(this,e,Fn,Gn,ne,{data:2,close:3})}}function dt(s,e,n){const t=s.slice();return t[12]=e[n],t}function _t(s,e,n){const t=s.slice();return t[15]=e[n][0],t[16]=e[n][1],t}function qn(s){let e,n,t,l;return{c(){e=c("details"),n=c("summary"),n.textContent="Full Muv JSON",t=y(),l=c("pre"),l.textContent=`${JSON.stringify(JSON.parse(s[0].muv),null,"  ")}`},m(o,r){g(o,e,r),f(e,n),f(e,t),f(e,l)},p:V,d(o){o&&h(e)}}}function gt(s){let e,n,t;return{c(){e=c("tr"),n=c("td"),n.textContent=`${s[15]}`,t=c("td"),t.textContent=`${s[16]}`,K(n,"class","svelte-860yh4"),K(t,"class","svelte-860yh4")},m(l,o){g(l,e,o),f(e,n),f(e,t)},p:V,d(l){l&&h(e)}}}function ht(s){let e,n,t=s[12]+"",l,o,r,u,i,a,b,O=he(Object.entries(JSON.parse(s[1].getOsmTagsForWay(BigInt(s[12]))))),T=[];for(let w=0;w<O.length;w+=1)T[w]=gt(_t(s,O,w));return{c(){e=c("p"),n=c("a"),l=R(t),o=y(),r=c("details"),u=c("summary"),u.textContent="See OSM tags",i=y(),a=c("table"),b=c("tbody");for(let w=0;w<T.length;w+=1)T[w].c();K(n,"href","https://www.openstreetmap.org/way/"+s[12]),K(n,"target","_blank")},m(w,I){g(w,e,I),f(e,n),f(n,l),g(w,o,I),g(w,r,I),f(r,u),f(r,i),f(r,a),f(a,b);for(let p=0;p<T.length;p+=1)T[p]&&T[p].m(b,null)},p(w,I){if(I&3){O=he(Object.entries(JSON.parse(w[1].getOsmTagsForWay(BigInt(w[12])))));let p;for(p=0;p<O.length;p+=1){const k=_t(w,O,p);T[p]?T[p].p(k,I):(T[p]=gt(k),T[p].c(),T[p].m(b,null))}for(;p<T.length;p+=1)T[p].d(1);T.length=O.length}},d(w){w&&(h(e),h(o),h(r)),Ge(T,w)}}}function Un(s){let e,n,t,l,o,r=s[0].type+"",u,i,a,b,O,T=s[0].direction+"",w,I,p,k,j,S=s[0].width+"",G,F,Y,D,W,U,E=s[0].speed_limit+"",$,Q,_,le,q,oe=s[0].allowed_turns+"",H,d,N,J,L,se=s[0].layer+"",x,ee,fe,pe,we,be,Ce,ge,ae,ce,Re,ke,m,de,_e,Ye,Be,Ve,Oe,ze,$e,Pe,Ke,xe,ye=s[0].muv&&qn(s),Le=he(JSON.parse(s[0].osm_way_ids)),te=[];for(let M=0;M<Le.length;M+=1)te[M]=ht(dt(s,Le,M));return{c(){e=c("h2"),e.textContent=`Lane ${s[0].index} of Road ${s[0].road}`,n=y(),t=c("p"),l=c("u"),l.textContent="Type",o=R(": "),u=R(r),i=y(),a=c("p"),b=c("u"),b.textContent="Direction",O=R(": "),w=R(T),I=y(),p=c("p"),k=c("u"),k.textContent="Width",j=R(": "),G=R(S),F=R("m"),Y=y(),D=c("p"),W=c("u"),W.textContent="Speed limit",U=R(": "),$=R(E),Q=y(),_=c("p"),le=c("u"),le.textContent="Allowed turns",q=R(": "),H=R(oe),d=y(),N=c("p"),J=c("u"),J.textContent="Layer",L=R(": "),x=R(se),ee=y(),ye&&ye.c(),fe=y(),pe=c("hr"),we=y(),be=c("p"),be.innerHTML="<u>OSM ways:</u>",Ce=y();for(let M=0;M<te.length;M+=1)te[M].c();ge=y(),ae=c("div"),ce=c("button"),ce.textContent="Collapse short road",Re=y(),ke=c("button"),ke.textContent="Zip side-path",m=y(),de=c("div"),_e=c("button"),_e.textContent="Find block on left",Ye=y(),Be=c("button"),Be.textContent="Find block on right",Ve=y(),Oe=c("div"),ze=c("button"),ze.textContent="Trace sidewalks on left",$e=y(),Pe=c("button"),Pe.textContent="Trace sidewalks on right",K(ce,"type","button"),K(ke,"type","button"),K(_e,"type","button"),K(Be,"type","button"),K(ze,"type","button"),K(Pe,"type","button")},m(M,A){g(M,e,A),g(M,n,A),g(M,t,A),f(t,l),f(t,o),f(t,u),g(M,i,A),g(M,a,A),f(a,b),f(a,O),f(a,w),g(M,I,A),g(M,p,A),f(p,k),f(p,j),f(p,G),f(p,F),g(M,Y,A),g(M,D,A),f(D,W),f(D,U),f(D,$),g(M,Q,A),g(M,_,A),f(_,le),f(_,q),f(_,H),g(M,d,A),g(M,N,A),f(N,J),f(N,L),f(N,x),g(M,ee,A),ye&&ye.m(M,A),g(M,fe,A),g(M,pe,A),g(M,we,A),g(M,be,A),g(M,Ce,A);for(let X=0;X<te.length;X+=1)te[X]&&te[X].m(M,A);g(M,ge,A),g(M,ae,A),f(ae,ce),f(ae,Re),f(ae,ke),g(M,m,A),g(M,de,A),f(de,_e),f(de,Ye),f(de,Be),g(M,Ve,A),g(M,Oe,A),f(Oe,ze),f(Oe,$e),f(Oe,Pe),Ke||(xe=[ue(ce,"click",s[2]),ue(ke,"click",s[3]),ue(_e,"click",s[7]),ue(Be,"click",s[8]),ue(ze,"click",s[9]),ue(Pe,"click",s[10])],Ke=!0)},p(M,[A]){if(M[0].muv&&ye.p(M,A),A&3){Le=he(JSON.parse(M[0].osm_way_ids));let X;for(X=0;X<Le.length;X+=1){const et=dt(M,Le,X);te[X]?te[X].p(et,A):(te[X]=ht(et),te[X].c(),te[X].m(ge.parentNode,ge))}for(;X<te.length;X+=1)te[X].d(1);te.length=Le.length}},i:V,o:V,d(M){M&&(h(e),h(n),h(t),h(i),h(a),h(I),h(p),h(Y),h(D),h(Q),h(_),h(d),h(N),h(ee),h(fe),h(pe),h(we),h(be),h(Ce),h(ge),h(ae),h(m),h(de),h(Ve),h(Oe)),ye&&ye.d(M),Ge(te,M),Ke=!1,It(xe)}}}function Vn(s,e,n){let t;Z(s,me,p=>n(11,t=p));let{data:l}=e,{close:o}=e,r=l.properties,u=t;function i(){t.collapseShortRoad(r.road),me.set(t),o()}function a(){t.zipSidepath(r.road),me.set(t),o()}function b(p,k){try{Je.set(JSON.parse(t.findBlock(r.road,p,k)))}catch(j){window.alert(j)}o()}const O=()=>b(!0,!1),T=()=>b(!1,!1),w=()=>b(!0,!0),I=()=>b(!1,!0);return s.$$set=p=>{"data"in p&&n(5,l=p.data),"close"in p&&n(6,o=p.close)},[r,u,i,a,b,l,o,O,T,w,I]}class Kn extends re{constructor(e){super(),ie(this,e,Vn,Un,ne,{data:5,close:6})}}function bt(s){let e,n;return e=new Kt({}),{c(){B(e.$$.fragment)},m(t,l){z(e,t,l),n=!0},i(t){n||(v(e.$$.fragment,t),n=!0)},o(t){C(e.$$.fragment,t),n=!1},d(t){P(e,t)}}}function Zn(s){let e,n,t,l,o,r,u,i,a,b,O,T,w,I,p,k,j;l=new qt({});let S=s[2]&&bt();return{c(){e=c("div"),n=c("h1"),n.textContent="osm2streets Street Explorer",t=y(),B(l.$$.fragment),o=y(),r=c("p"),r.innerHTML=`Understanding OSM streets &amp; intersections with
      <a href="https://github.com/a-b-street/osm2streets" target="_blank">osm2streets</a>
      and <a href="https://gitlab.com/LeLuxNet/Muv" target="_blank">Muv</a>.`,u=y(),i=c("hr"),a=y(),S&&S.c(),b=y(),O=c("br"),T=y(),w=c("details"),I=c("summary"),I.textContent="Layers",p=y(),k=c("div"),w.open=!0,K(w,"class","svelte-1n0zlav"),K(e,"slot","left")},m(G,F){g(G,e,F),f(e,n),f(e,t),z(l,e,null),f(e,o),f(e,r),f(e,u),f(e,i),f(e,a),S&&S.m(e,null),f(e,b),f(e,O),f(e,T),f(e,w),f(w,I),f(w,p),f(w,k),s[4](k),j=!0},p(G,F){G[2]?S?F&4&&v(S,1):(S=bt(),S.c(),v(S,1),S.m(e,b)):S&&(Ie(),C(S,1,1,()=>{S=null}),je())},i(G){j||(v(l.$$.fragment,G),v(S),j=!0)},o(G){C(l.$$.fragment,G),C(S),j=!1},d(G){G&&h(e),P(l),S&&S.d(),s[4](null)}}}function kt(s){let e,n;return e=new Wn({props:{data:s[5],close:s[6]}}),{c(){B(e.$$.fragment)},m(t,l){z(e,t,l),n=!0},p(t,l){const o={};l&32&&(o.data=t[5]),l&64&&(o.close=t[6]),e.$set(o)},i(t){n||(v(e.$$.fragment,t),n=!0)},o(t){C(e.$$.fragment,t),n=!1},d(t){P(e,t)}}}function Qn(s){let e=s[5],n,t,l=kt(s);return{c(){l.c(),n=De()},m(o,r){l.m(o,r),g(o,n,r),t=!0},p(o,r){r&32&&ne(e,e=o[5])?(Ie(),C(l,1,1,V),je(),l=kt(o),l.c(),v(l,1),l.m(n.parentNode,n)):l.p(o,r)},i(o){t||(v(l),t=!0)},o(o){C(l),t=!1},d(o){o&&h(n),l.d(o)}}}function Xn(s){let e,n;return e=new Xe({props:{openOn:"click",popupClass:"popup",$$slots:{default:[Qn,({data:t,close:l})=>({5:t,6:l}),({data:t,close:l})=>(t?32:0)|(l?64:0)]},$$scope:{ctx:s}}}),{c(){B(e.$$.fragment)},m(t,l){z(e,t,l),n=!0},p(t,l){const o={};l&224&&(o.$$scope={dirty:l,ctx:t}),e.$set(o)},i(t){n||(v(e.$$.fragment,t),n=!0)},o(t){C(e.$$.fragment,t),n=!1},d(t){P(e,t)}}}function yt(s){let e,n;return e=new Kn({props:{data:s[5],close:s[6]}}),{c(){B(e.$$.fragment)},m(t,l){z(e,t,l),n=!0},p(t,l){const o={};l&32&&(o.data=t[5]),l&64&&(o.close=t[6]),e.$set(o)},i(t){n||(v(e.$$.fragment,t),n=!0)},o(t){C(e.$$.fragment,t),n=!1},d(t){P(e,t)}}}function Yn(s){let e=s[5],n,t,l=yt(s);return{c(){l.c(),n=De()},m(o,r){l.m(o,r),g(o,n,r),t=!0},p(o,r){r&32&&ne(e,e=o[5])?(Ie(),C(l,1,1,V),je(),l=yt(o),l.c(),v(l,1),l.m(n.parentNode,n)):l.p(o,r)},i(o){t||(v(l),t=!0)},o(o){C(l),t=!1},d(o){o&&h(n),l.d(o)}}}function $n(s){let e,n;return e=new Xe({props:{openOn:"click",popupClass:"popup",$$slots:{default:[Yn,({data:t,close:l})=>({5:t,6:l}),({data:t,close:l})=>(t?32:0)|(l?64:0)]},$$scope:{ctx:s}}}),{c(){B(e.$$.fragment)},m(t,l){z(e,t,l),n=!0},p(t,l){const o={};l&224&&(o.$$scope={dirty:l,ctx:t}),e.$set(o)},i(t){n||(v(e.$$.fragment,t),n=!0)},o(t){C(e.$$.fragment,t),n=!1},d(t){P(e,t)}}}function xn(s){let e,n,t,l,o,r,u,i,a,b,O,T,w,I,p,k,j,S,G,F,Y,D,W,U,E,$,Q,_,le,q,oe,H,d,N,J;return n=new Zt({}),l=new Qt({props:{hoverCursor:"pointer",$$slots:{default:[Xn]},$$scope:{ctx:s}}}),r=new Xt({}),i=new Yt({props:{hoverCursor:"pointer",$$slots:{default:[$n]},$$scope:{ctx:s}}}),b=new $t({}),I=new Cn({}),S=new Nn({}),F=new Pn({}),D=new Tn({}),U=new Dn({}),_=new cn({}),q=new hn({}),H=new xt({}),N=new en({}),{c(){e=c("div"),B(n.$$.fragment),t=y(),B(l.$$.fragment),o=y(),B(r.$$.fragment),u=y(),B(i.$$.fragment),a=y(),B(b.$$.fragment),O=y(),T=c("hr"),w=y(),B(I.$$.fragment),p=y(),k=c("hr"),j=y(),B(S.$$.fragment),G=y(),B(F.$$.fragment),Y=y(),B(D.$$.fragment),W=y(),B(U.$$.fragment),E=y(),$=c("hr"),Q=y(),B(_.$$.fragment),le=y(),B(q.$$.fragment),oe=y(),B(H.$$.fragment),d=y(),B(N.$$.fragment)},m(L,se){g(L,e,se),z(n,e,null),f(e,t),z(l,e,null),f(e,o),z(r,e,null),f(e,u),z(i,e,null),f(e,a),z(b,e,null),f(e,O),f(e,T),f(e,w),z(I,e,null),f(e,p),f(e,k),f(e,j),z(S,e,null),f(e,G),z(F,e,null),f(e,Y),z(D,e,null),f(e,W),z(U,e,null),f(e,E),f(e,$),f(e,Q),z(_,e,null),f(e,le),z(q,e,null),f(e,oe),z(H,e,null),s[3](e),g(L,d,se),z(N,L,se),J=!0},p(L,se){const x={};se&128&&(x.$$scope={dirty:se,ctx:L}),l.$set(x);const ee={};se&128&&(ee.$$scope={dirty:se,ctx:L}),i.$set(ee)},i(L){J||(v(n.$$.fragment,L),v(l.$$.fragment,L),v(r.$$.fragment,L),v(i.$$.fragment,L),v(b.$$.fragment,L),v(I.$$.fragment,L),v(S.$$.fragment,L),v(F.$$.fragment,L),v(D.$$.fragment,L),v(U.$$.fragment,L),v(_.$$.fragment,L),v(q.$$.fragment,L),v(H.$$.fragment,L),v(N.$$.fragment,L),J=!0)},o(L){C(n.$$.fragment,L),C(l.$$.fragment,L),C(r.$$.fragment,L),C(i.$$.fragment,L),C(b.$$.fragment,L),C(I.$$.fragment,L),C(S.$$.fragment,L),C(F.$$.fragment,L),C(D.$$.fragment,L),C(U.$$.fragment,L),C(_.$$.fragment,L),C(q.$$.fragment,L),C(H.$$.fragment,L),C(N.$$.fragment,L),J=!1},d(L){L&&(h(e),h(d)),P(n),P(l),P(r),P(i),P(b),P(I),P(S),P(F),P(D),P(U),P(_),P(q),P(H),s[3](null),P(N,L)}}}function el(s){let e,n,t;return n=new Ut({props:{$$slots:{default:[xn]},$$scope:{ctx:s}}}),{c(){e=c("div"),B(n.$$.fragment),K(e,"slot","main")},m(l,o){g(l,e,o),z(n,e,null),t=!0},p(l,o){const r={};o&129&&(r.$$scope={dirty:o,ctx:l}),n.$set(r)},i(l){t||(v(n.$$.fragment,l),t=!0)},o(l){C(n.$$.fragment,l),t=!1},d(l){l&&h(e),P(n)}}}function tl(s){let e,n;return e=new Wt({props:{$$slots:{main:[el],left:[Zn]},$$scope:{ctx:s}}}),{c(){B(e.$$.fragment)},m(t,l){z(e,t,l),n=!0},p(t,[l]){const o={};l&135&&(o.$$scope={dirty:l,ctx:t}),e.$set(o)},i(t){n||(v(e.$$.fragment,t),n=!0)},o(t){C(e.$$.fragment,t),n=!1},d(t){P(e,t)}}}function nl(s,e,n){let t=!1;vt(async()=>{await Vt(),n(2,t=!0)});let l=null,o;function r(i){ve[i?"unshift":"push"](()=>{l=i,n(0,l)})}function u(i){ve[i?"unshift":"push"](()=>{o=i,n(1,o),n(0,l)})}return s.$$.update=()=>{s.$$.dirty&3&&l&&o&&(n(1,o.innerHTML="",o),o.appendChild(l))},[l,o,t,r,u]}class ll extends re{constructor(e){super(),ie(this,e,nl,tl,ne,{})}}new ll({target:document.getElementById("app")});