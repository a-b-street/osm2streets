import{S as se,i as re,s as x,e as He,a as g,t as w,g as Ie,b as L,c as je,d as h,f as zt,h as Pt,m as Rt,j as U,o as Ot,k as Se,l as Et,n as At,p as c,q as Lt,u as Mt,r as St,v as Tt,w as Dt,x as ve,L as Ht,y as Fe,z as J,A as z,B as Ge,C as P,D as Ft,E as Me,F as ge,G as V,H as $e,I as v,J as R,K as q,M as lt,N as a,O as Qe,P as Te,Q as It,R as De,T as oe,U as ot,V as jt,W as st,X as ae,Y as Ne,Z as Nt,_ as We,$ as Bt,a0 as Re,a1 as rt,a2 as it,a3 as Ee,a4 as Xe,a5 as Gt,a6 as Ue,a7 as Ye,a8 as Jt,a9 as $t,aa as Wt,ab as Ut,ac as qt,ad as Vt,ae as Kt,af as Zt,ag as Qt,ah as Xt,ai as Yt,aj as xt,ak as en,al as tn,am as nn}from"./RenderLanePolygons-0649c17b.js";const ln=s=>({features:s[0]&16,data:s[0]&16,map:s[0]&4,close:s[0]&1}),ut=s=>{var e;return{features:s[4],data:(e=s[4])==null?void 0:e[0],map:s[2],close:s[31]}};function ft(s){let e,n,t=(s[4]||s[3]instanceof Se.Marker)&&at(s);return{c(){e=c("div"),t&&t.c()},m(l,o){g(l,e,o),t&&t.m(e,null),s[32](e),n=!0},p(l,o){l[4]||l[3]instanceof Se.Marker?t?(t.p(l,o),o[0]&24&&w(t,1)):(t=at(l),t.c(),w(t,1),t.m(e,null)):t&&(Ie(),L(t,1,1,()=>{t=null}),je())},i(l){n||(w(t),n=!0)},o(l){L(t),n=!1},d(l){l&&h(e),t&&t.d(),s[32](null)}}}function at(s){let e;const n=s[30].default,t=Lt(n,s,s[29],ut);return{c(){t&&t.c()},m(l,o){t&&t.m(l,o),e=!0},p(l,o){t&&t.p&&(!e||o[0]&536870933)&&Mt(t,n,l,l[29],e?Tt(n,l[29],o,ln):St(l[29]),ut)},i(l){e||(w(t,l),e=!0)},o(l){L(t,l),e=!1},d(l){t&&t.d(l)}}}function on(s){let e,n,t=s[9].default&&ft(s);return{c(){t&&t.c(),e=He()},m(l,o){t&&t.m(l,o),g(l,e,o),n=!0},p(l,o){l[9].default?t?(t.p(l,o),o[0]&512&&w(t,1)):(t=ft(l),t.c(),w(t,1),t.m(e.parentNode,e)):t&&(Ie(),L(t,1,1,()=>{t=null}),je())},i(l){n||(w(t),n=!0)},o(l){L(t),n=!1},d(l){l&&h(e),t&&t.d(l)}}}function sn(s,e,n){let t,l,o,r,u,i,{$$slots:f={},$$scope:m}=e;const O=zt(f);let{closeButton:S=void 0}=e,{closeOnClickOutside:C=!0}=e,{closeOnClickInside:M=!1}=e,{closeOnMove:y=!1}=e,{openOn:j="click"}=e,{openIfTopMost:I=!0}=e,{focusAfterOpen:k=!0}=e,{anchor:N=void 0}=e,{offset:A=void 0}=e,{popupClass:Q=void 0}=e,{maxWidth:D=void 0}=e,{lngLat:G=void 0}=e,{html:W=void 0}=e,{open:H=!1}=e;const X=Pt(),{map:K,popupTarget:_,layerEvent:ee,layer:$,eventTopMost:te}=Rt();U(s,K,p=>n(2,o=p)),U(s,_,p=>n(3,u=p)),U(s,ee,p=>n(28,r=p)),U(s,$,p=>n(35,i=p));const F=["click","dblclick","contextmenu"];let d,b=!1,B;function ce(){if(!d)return;let p=d.getElement();!p||p===B||(B=p,j==="hover"&&(B.style.pointerEvents="none"),B.addEventListener("mouseenter",()=>{n(24,b=!0)},{passive:!0}),B.addEventListener("mouseleave",()=>{n(24,b=!1)},{passive:!0}),B.addEventListener("click",()=>{M&&n(0,H=!1)},{passive:!0}))}Ot(()=>{if(o)return o.on("click",ue),o.on("contextmenu",ue),typeof u=="string"&&(o.on("click",u,ie),o.on("dblclick",u,ie),o.on("contextmenu",u,ie),o.on("mousemove",u,_e),o.on("mouseleave",u,Ce),o.on("touchstart",u,we),o.on("touchend",u,be)),()=>{o!=null&&o.loaded()&&(d==null||d.remove(),o.off("click",ue),o.off("contextmenu",ue),u instanceof Se.Marker?u.getPopup()===d&&u.setPopup(void 0):typeof u=="string"&&(o.off("click",u,ie),o.off("dblclick",u,ie),o.off("contextmenu",u,ie),o.off("mousemove",u,_e),o.off("mouseleave",u,Ce),o.off("touchstart",u,we),o.off("touchend",u,be)))}});function he(p){return I?!("marker"in p)&&!Dt(p)&&te(p)!==i:!1}let ne=null,le="normal";function ie(p){p.type===j&&(he(p)||("layerType"in p?p.layerType==="deckgl"?(n(10,G=p.coordinate),n(4,ne=p.object?[p.object]:null)):(n(10,G=p.lngLat),n(4,ne=p.features??[])):(n(10,G=p.lngLat),n(4,ne=p.features??[])),setTimeout(()=>n(0,H=!0))))}let pe=null;function we(p){pe=p.point}function be(p){if(!pe||j!=="hover")return;let me=pe.dist(p.point);pe=null,me<3&&(n(10,G=p.lngLat),n(4,ne=p.features??[]),d.isOpen()?n(25,le="justOpened"):(n(25,le="opening"),n(0,H=!0)))}function Ce(p){j!=="hover"||pe||le!=="normal"||(n(0,H=!1),n(4,ne=null))}function _e(p){if(!(j!=="hover"||pe||le!=="normal")){if(he(p)){n(0,H=!1),n(4,ne=null);return}n(0,H=!0),n(4,ne=p.features??[]),n(10,G=p.lngLat)}}function ue(p){if(le==="justOpened"){n(25,le="normal");return}if(!C)return;let me=[B,u instanceof Se.Marker?u==null?void 0:u.getElement():null];H&&d.isOpen()&&!me.some(de=>de==null?void 0:de.contains(p.originalEvent.target))&&(p.type==="contextmenu"&&j==="contextmenu"||p.type!=="contextmenu")&&n(0,H=!1)}Et(()=>{o&&(d!=null&&d.isOpen())&&d.remove()});let fe;const Ae=()=>n(0,H=!1);function ke(p){ve[p?"unshift":"push"](()=>{fe=p,n(1,fe)})}return s.$$set=p=>{"closeButton"in p&&n(11,S=p.closeButton),"closeOnClickOutside"in p&&n(12,C=p.closeOnClickOutside),"closeOnClickInside"in p&&n(13,M=p.closeOnClickInside),"closeOnMove"in p&&n(14,y=p.closeOnMove),"openOn"in p&&n(15,j=p.openOn),"openIfTopMost"in p&&n(16,I=p.openIfTopMost),"focusAfterOpen"in p&&n(17,k=p.focusAfterOpen),"anchor"in p&&n(18,N=p.anchor),"offset"in p&&n(19,A=p.offset),"popupClass"in p&&n(20,Q=p.popupClass),"maxWidth"in p&&n(21,D=p.maxWidth),"lngLat"in p&&n(10,G=p.lngLat),"html"in p&&n(22,W=p.html),"open"in p&&n(0,H=p.open),"$$scope"in p&&n(29,m=p.$$scope)},s.$$.update=()=>{if(s.$$.dirty[0]&14336&&n(27,t=S??(!C&&!M)),s.$$.dirty[0]&146685952&&(d||(n(23,d=new Se.Popup({closeButton:t,closeOnClick:!1,closeOnMove:y,focusAfterOpen:k,maxWidth:D,className:Q,anchor:N,offset:A})),B=d.getElement(),d.on("open",()=>{n(0,H=!0),ce(),X("open",d)}),d.on("close",()=>{n(0,H=!1),X("close",d)}),d.on("hover",()=>{X("hover",d)}))),s.$$.dirty[0]&8421384&&d&&u instanceof Se.Marker&&(j==="click"?u.setPopup(d):u.getPopup()===d&&u.setPopup(void 0)),s.$$.dirty[0]&268468224&&F.includes(j)&&(r==null?void 0:r.type)===j&&(ie(r),At(ee,r=null,r)),s.$$.dirty[0]&268468224&&n(26,l=j==="hover"&&((r==null?void 0:r.type)==="mousemove"||(r==null?void 0:r.type)==="mouseenter")),s.$$.dirty[0]&352354304&&j==="hover"&&ee&&(l&&r&&(r.layerType==="deckgl"?(n(10,G=r.coordinate),n(4,ne=r.object?[r.object]:null)):(n(10,G=r.lngLat),n(4,ne=r.features??[]))),n(0,H=(l||b)??!1)),s.$$.dirty[0]&12582914&&(fe?d.setDOMContent(fe):W&&d.setHTML(W)),s.$$.dirty[0]&8389632&&G&&d.setLngLat(G),s.$$.dirty[0]&41943045&&o){let p=d.isOpen();H&&!p?(d.addTo(o),le==="opening"&&n(25,le="justOpened")):!H&&p&&d.remove()}},[H,fe,o,u,ne,K,_,ee,$,O,G,S,C,M,y,j,I,k,N,A,Q,D,W,d,b,le,l,t,r,m,f,Ae,ke]}class qe extends se{constructor(e){super(),re(this,e,sn,on,x,{closeButton:11,closeOnClickOutside:12,closeOnClickInside:13,closeOnMove:14,openOn:15,openIfTopMost:16,focusAfterOpen:17,anchor:18,offset:19,popupClass:20,maxWidth:21,lngLat:10,html:22,open:0},null,[-1,-1])}}function rn(s){let e;const n=s[16].default,t=Lt(n,s,s[24],null);return{c(){t&&t.c()},m(l,o){t&&t.m(l,o),e=!0},p(l,o){t&&t.p&&(!e||o&16777216)&&Mt(t,n,l,l[24],e?Tt(n,l[24],o,null):St(l[24]),null)},i(l){e||(w(t,l),e=!0)},o(l){L(t,l),e=!1},d(l){t&&t.d(l)}}}function un(s){let e,n,t;function l(r){s[17](r)}let o={id:s[1],type:"symbol",source:s[2],sourceLayer:s[3],beforeId:s[4],beforeLayerType:s[5],paint:s[6],layout:s[7],filter:s[8],applyToClusters:s[9],minzoom:s[10],maxzoom:s[11],hoverCursor:s[12],manageHoverState:s[13],eventsIfTopMost:s[14],interactive:s[15],$$slots:{default:[rn]},$$scope:{ctx:s}};return s[0]!==void 0&&(o.hovered=s[0]),e=new Ht({props:o}),ve.push(()=>Fe(e,"hovered",l)),e.$on("click",s[18]),e.$on("dblclick",s[19]),e.$on("contextmenu",s[20]),e.$on("mouseenter",s[21]),e.$on("mousemove",s[22]),e.$on("mouseleave",s[23]),{c(){J(e.$$.fragment)},m(r,u){z(e,r,u),t=!0},p(r,[u]){const i={};u&2&&(i.id=r[1]),u&4&&(i.source=r[2]),u&8&&(i.sourceLayer=r[3]),u&16&&(i.beforeId=r[4]),u&32&&(i.beforeLayerType=r[5]),u&64&&(i.paint=r[6]),u&128&&(i.layout=r[7]),u&256&&(i.filter=r[8]),u&512&&(i.applyToClusters=r[9]),u&1024&&(i.minzoom=r[10]),u&2048&&(i.maxzoom=r[11]),u&4096&&(i.hoverCursor=r[12]),u&8192&&(i.manageHoverState=r[13]),u&16384&&(i.eventsIfTopMost=r[14]),u&32768&&(i.interactive=r[15]),u&16777216&&(i.$$scope={dirty:u,ctx:r}),!n&&u&1&&(n=!0,i.hovered=r[0],Ge(()=>n=!1)),e.$set(i)},i(r){t||(w(e.$$.fragment,r),t=!0)},o(r){L(e.$$.fragment,r),t=!1},d(r){P(e,r)}}}function fn(s,e,n){let{$$slots:t={},$$scope:l}=e,{id:o=Ft("symbol")}=e,{source:r=void 0}=e,{sourceLayer:u=void 0}=e,{beforeId:i=void 0}=e,{beforeLayerType:f=void 0}=e,{paint:m=void 0}=e,{layout:O=void 0}=e,{filter:S=void 0}=e,{applyToClusters:C=void 0}=e,{minzoom:M=void 0}=e,{maxzoom:y=void 0}=e,{hoverCursor:j=void 0}=e,{manageHoverState:I=!1}=e,{hovered:k=null}=e,{eventsIfTopMost:N=!1}=e,{interactive:A=!0}=e;function Q(_){k=_,n(0,k)}function D(_){Me.call(this,s,_)}function G(_){Me.call(this,s,_)}function W(_){Me.call(this,s,_)}function H(_){Me.call(this,s,_)}function X(_){Me.call(this,s,_)}function K(_){Me.call(this,s,_)}return s.$$set=_=>{"id"in _&&n(1,o=_.id),"source"in _&&n(2,r=_.source),"sourceLayer"in _&&n(3,u=_.sourceLayer),"beforeId"in _&&n(4,i=_.beforeId),"beforeLayerType"in _&&n(5,f=_.beforeLayerType),"paint"in _&&n(6,m=_.paint),"layout"in _&&n(7,O=_.layout),"filter"in _&&n(8,S=_.filter),"applyToClusters"in _&&n(9,C=_.applyToClusters),"minzoom"in _&&n(10,M=_.minzoom),"maxzoom"in _&&n(11,y=_.maxzoom),"hoverCursor"in _&&n(12,j=_.hoverCursor),"manageHoverState"in _&&n(13,I=_.manageHoverState),"hovered"in _&&n(0,k=_.hovered),"eventsIfTopMost"in _&&n(14,N=_.eventsIfTopMost),"interactive"in _&&n(15,A=_.interactive),"$$scope"in _&&n(24,l=_.$$scope)},[k,o,r,u,i,f,m,O,S,C,M,y,j,I,N,A,t,Q,D,G,W,H,X,K,l]}class an extends se{constructor(e){super(),re(this,e,fn,un,x,{id:1,source:2,sourceLayer:3,beforeId:4,beforeLayerType:5,paint:6,layout:7,filter:8,applyToClusters:9,minzoom:10,maxzoom:11,hoverCursor:12,manageHoverState:13,hovered:0,eventsIfTopMost:14,interactive:15})}}function ct(s,e,n){const t=s.slice();return t[1]=e[n][0],t[2]=e[n][1],t}function pt(s){let e,n,t,l=s[1]+"",o,r;return{c(){e=c("li"),n=c("span"),t=v(),o=R(l),r=v(),q(n,"class","svelte-kzgqtg"),lt(n,"background",s[2])},m(u,i){g(u,e,i),a(e,n),a(e,t),a(e,o),a(e,r)},p(u,i){i&1&&lt(n,"background",u[2]),i&1&&l!==(l=u[1]+"")&&Qe(o,l)},d(u){u&&h(e)}}}function cn(s){let e,n=ge(s[0]),t=[];for(let l=0;l<n.length;l+=1)t[l]=pt(ct(s,n,l));return{c(){e=c("ul");for(let l=0;l<t.length;l+=1)t[l].c()},m(l,o){g(l,e,o);for(let r=0;r<t.length;r+=1)t[r]&&t[r].m(e,null)},p(l,[o]){if(o&1){n=ge(l[0]);let r;for(r=0;r<n.length;r+=1){const u=ct(l,n,r);t[r]?t[r].p(u,o):(t[r]=pt(u),t[r].c(),t[r].m(e,null))}for(;r<t.length;r+=1)t[r].d(1);t.length=n.length}},i:V,o:V,d(l){l&&h(e),$e(t,l)}}}function pn(s,e,n){let{rows:t}=e;return s.$$set=l=>{"rows"in l&&n(0,t=l.rows)},[t]}class mn extends se{constructor(e){super(),re(this,e,pn,cn,x,{rows:0})}}function dn(s){let e,n,t,l,o,r,u,i,f,m;return{c(){e=c("div"),n=c("label"),t=R(`Basemap:
    `),l=c("select"),o=c("option"),o.textContent="MapTiler Dataviz",r=c("option"),r.textContent="MapTiler Streets",u=c("option"),u.textContent="MapTiler Satellite",i=c("option"),i.textContent="Blank",o.__value="dataviz",Te(o,o.__value),r.__value="streets",Te(r,r.__value),u.__value="hybrid",Te(u,u.__value),i.__value="blank",Te(i,i.__value),s[0]===void 0&&It(()=>s[1].call(l))},m(O,S){g(O,e,S),a(e,n),a(n,t),a(n,l),a(l,o),a(l,r),a(l,u),a(l,i),De(l,s[0],!0),f||(m=oe(l,"change",s[1]),f=!0)},p(O,[S]){S&1&&De(l,O[0])},i:V,o:V,d(O){O&&h(e),f=!1,m()}}}function _n(s,e,n){let t;U(s,ot,o=>n(0,t=o));function l(){t=jt(this),ot.set(t)}return[t,l]}class gn extends se{constructor(e){super(),re(this,e,_n,dn,x,{})}}function hn(s){let e,n,t,l,o,r,u,i;return{c(){e=c("div"),n=c("label"),t=R(`Theme:
    `),l=c("select"),o=c("option"),o.textContent="Debug",r=c("option"),r.textContent="Realistic",o.__value="debug",Te(o,o.__value),r.__value="realistic",Te(r,r.__value),s[0]===void 0&&It(()=>s[1].call(l))},m(f,m){g(f,e,m),a(e,n),a(n,t),a(n,l),a(l,o),a(l,r),De(l,s[0],!0),u||(i=oe(l,"change",s[1]),u=!0)},p(f,[m]){m&1&&De(l,f[0])},i:V,o:V,d(f){f&&h(e),u=!1,i()}}}function bn(s,e,n){let t;U(s,st,o=>n(0,t=o));function l(){t=jt(this),st.set(t)}return[t,l]}class kn extends se{constructor(e){super(),re(this,e,bn,hn,x,{})}}const Pe=Nt(Ne()),Ze=Nt(!1);ae.subscribe(s=>{Pe.set(Ne())});function yn(s){let e,n=s[11].properties.kind+"",t;return{c(){e=c("p"),t=R(n)},m(l,o){g(l,e,o),a(e,t)},p(l,o){o&2048&&n!==(n=l[11].properties.kind+"")&&Qe(t,n)},d(l){l&&h(e)}}}function vn(s){let e,n;return e=new qe({props:{openOn:"hover",$$slots:{default:[yn,({data:t})=>({11:t}),({data:t})=>t?2048:0]},$$scope:{ctx:s}}}),{c(){J(e.$$.fragment)},m(t,l){z(e,t,l),n=!0},p(t,l){const o={};l&6144&&(o.$$scope={dirty:l,ctx:t}),e.$set(o)},i(t){n||(w(e.$$.fragment,t),n=!0)},o(t){L(e.$$.fragment,t),n=!1},d(t){P(e,t)}}}function wn(s){let e,n=JSON.stringify(s[11].properties,null,"  ")+"",t;return{c(){e=c("pre"),t=R(n)},m(l,o){g(l,e,o),a(e,t)},p(l,o){o&2048&&n!==(n=JSON.stringify(l[11].properties,null,"  ")+"")&&Qe(t,n)},d(l){l&&h(e)}}}function Cn(s){let e,n;return e=new qe({props:{openOn:"hover",$$slots:{default:[wn,({data:t})=>({11:t}),({data:t})=>t?2048:0]},$$scope:{ctx:s}}}),{c(){J(e.$$.fragment)},m(t,l){z(e,t,l),n=!0},p(t,l){const o={};l&6144&&(o.$$scope={dirty:l,ctx:t}),e.$set(o)},i(t){n||(w(e.$$.fragment,t),n=!0)},o(t){L(e.$$.fragment,t),n=!1},d(t){P(e,t)}}}function On(s){let e,n,t,l;const o=[Re("block"),{filter:["==",["get","type"],"block"]},{manageHoverState:!0},{paint:{"fill-color":rt(["get","kind"],s[1],"red"),"fill-opacity":it(.8,.4)}}];let r={$$slots:{default:[vn]},$$scope:{ctx:s}};for(let f=0;f<o.length;f+=1)r=Ee(r,o[f]);e=new Xe({props:r});const u=[Re("block-debug"),{filter:["!=",["get","type"],"block"]},{paint:{"line-color":["case",["==",["get","type"],"member-road"],"red","black"],"line-width":5}}];let i={$$slots:{default:[Cn]},$$scope:{ctx:s}};for(let f=0;f<u.length;f+=1)i=Ee(i,u[f]);return t=new Gt({props:i}),{c(){J(e.$$.fragment),n=v(),J(t.$$.fragment)},m(f,m){z(e,f,m),g(f,n,m),z(t,f,m),l=!0},p(f,m){const O=m&2?Ue(o,[o[0],o[1],o[2],{paint:{"fill-color":rt(["get","kind"],f[1],"red"),"fill-opacity":it(.8,.4)}}]):{};m&4096&&(O.$$scope={dirty:m,ctx:f}),e.$set(O);const S={};m&4096&&(S.$$scope={dirty:m,ctx:f}),t.$set(S)},i(f){l||(w(e.$$.fragment,f),w(t.$$.fragment,f),l=!0)},o(f){L(e.$$.fragment,f),L(t.$$.fragment,f),l=!1},d(f){f&&h(n),P(e,f),P(t,f)}}}function mt(s){let e,n;return e=new mn({props:{rows:Object.entries(s[1])}}),{c(){J(e.$$.fragment)},m(t,l){z(e,t,l),n=!0},p(t,l){const o={};l&2&&(o.rows=Object.entries(t[1])),e.$set(o)},i(t){n||(w(e.$$.fragment,t),n=!0)},o(t){L(e.$$.fragment,t),n=!1},d(t){P(e,t)}}}function Ln(s){let e,n,t,l,o,r,u,i,f,m,O,S,C,M,y,j;e=new We({props:{data:s[0],generateId:!0,$$slots:{default:[On]},$$scope:{ctx:s}}});let I=s[2]&&mt(s);return{c(){J(e.$$.fragment),n=v(),t=c("div"),l=R(`Blocks
  `),o=c("button"),r=R("Clear"),i=v(),f=c("button"),f.textContent="Find all blocks",m=v(),O=c("button"),O.textContent="Find all sidewalk bundles",S=v(),I&&I.c(),C=He(),o.disabled=u=!s[2]},m(k,N){z(e,k,N),g(k,n,N),g(k,t,N),a(t,l),a(t,o),a(o,r),a(t,i),a(t,f),a(t,m),a(t,O),g(k,S,N),I&&I.m(k,N),g(k,C,N),M=!0,y||(j=[oe(o,"click",s[3]),oe(f,"click",s[6]),oe(O,"click",s[7])],y=!0)},p(k,[N]){const A={};N&1&&(A.data=k[0]),N&4098&&(A.$$scope={dirty:N,ctx:k}),e.$set(A),(!M||N&4&&u!==(u=!k[2]))&&(o.disabled=u),k[2]?I?(I.p(k,N),N&4&&w(I,1)):(I=mt(k),I.c(),w(I,1),I.m(C.parentNode,C)):I&&(Ie(),L(I,1,1,()=>{I=null}),je())},i(k){M||(w(e.$$.fragment,k),w(I),M=!0)},o(k){L(e.$$.fragment,k),L(I),M=!1},d(k){k&&(h(n),h(t),h(S),h(C)),P(e,k),I&&I.d(k),y=!1,Bt(j)}}}function Mn(s,e,n){let t,l,o,r,u;U(s,Ze,M=>n(5,o=M)),U(s,ae,M=>n(8,r=M)),U(s,Pe,M=>n(0,u=M));function i(){Pe.set(Ne())}function f(M){Pe.set(JSON.parse(r.findAllBlocks(M))),Ze.set(M)}let m={LandUseBlock:"grey",RoadAndSidewalk:"green",RoadAndCycleLane:"orange",CycleLaneAndSidewalk:"yellow",DualCarriageway:"purple",Unknown:"blue"},O={LandUseBlock:"grey",RoadBundle:"green",IntersectionBundle:"orange",Unknown:"blue"};const S=()=>f(!1),C=()=>f(!0);return s.$$.update=()=>{s.$$.dirty&1&&n(2,t=u.features.length>0),s.$$.dirty&32&&n(1,l=o?O:m)},[u,l,t,i,f,o,S,C]}class Sn extends se{constructor(e){super(),re(this,e,Mn,Ln,x,{})}}function Tn(s){let e,n;const t=[Re("connected-roads"),{layout:{visibility:s[0]?"visible":"none"}},{paint:{"fill-color":"blue","fill-opacity":.5}}];let l={};for(let o=0;o<t.length;o+=1)l=Ee(l,t[o]);return e=new Xe({props:l}),{c(){J(e.$$.fragment)},m(o,r){z(e,o,r),n=!0},p(o,r){const u=r&1?Ue(t,[t[0],{layout:{visibility:o[0]?"visible":"none"}},t[2]]):{};e.$set(u)},i(o){n||(w(e.$$.fragment,o),n=!0)},o(o){L(e.$$.fragment,o),n=!1},d(o){P(e,o)}}}function In(s){let e,n,t,l,o;e=new We({props:{data:s[1],$$slots:{default:[Tn]},$$scope:{ctx:s}}});function r(i){s[4](i)}let u={gj:s[1],name:"Roads connected to intersection",downloadable:!1};return s[0]!==void 0&&(u.show=s[0]),t=new Ye({props:u}),ve.push(()=>Fe(t,"show",r)),{c(){J(e.$$.fragment),n=v(),J(t.$$.fragment)},m(i,f){z(e,i,f),g(i,n,f),z(t,i,f),o=!0},p(i,[f]){const m={};f&2&&(m.data=i[1]),f&33&&(m.$$scope={dirty:f,ctx:i}),e.$set(m);const O={};f&2&&(O.gj=i[1]),!l&&f&1&&(l=!0,O.show=i[0],Ge(()=>l=!1)),t.$set(O)},i(i){o||(w(e.$$.fragment,i),w(t.$$.fragment,i),o=!0)},o(i){L(e.$$.fragment,i),L(t.$$.fragment,i),o=!1},d(i){i&&h(n),P(e,i),P(t,i)}}}function jn(s,e,n){let t,l,o;U(s,Jt,i=>n(2,l=i)),U(s,ae,i=>n(3,o=i));let r=!1;function u(i){r=i,n(0,r)}return s.$$.update=()=>{s.$$.dirty&12&&n(1,t=o&&l?JSON.parse(o.debugRoadsConnectedToIntersectionGeojson(l.properties.id)):Ne())},[r,t,l,o,u]}class Nn extends se{constructor(e){super(),re(this,e,jn,In,x,{})}}function Bn(s){let e,n;const t=[Re("movements"),{layout:{visibility:s[0]?"visible":"none"}},{paint:{"fill-color":"blue","fill-opacity":.5}}];let l={};for(let o=0;o<t.length;o+=1)l=Ee(l,t[o]);return e=new Xe({props:l}),{c(){J(e.$$.fragment)},m(o,r){z(e,o,r),n=!0},p(o,r){const u=r&1?Ue(t,[t[0],{layout:{visibility:o[0]?"visible":"none"}},t[2]]):{};e.$set(u)},i(o){n||(w(e.$$.fragment,o),n=!0)},o(o){L(e.$$.fragment,o),n=!1},d(o){P(e,o)}}}function Jn(s){let e,n,t,l,o;e=new We({props:{data:s[1],$$slots:{default:[Bn]},$$scope:{ctx:s}}});function r(i){s[4](i)}let u={gj:s[1],name:"Movement arrows",downloadable:!1};return s[0]!==void 0&&(u.show=s[0]),t=new Ye({props:u}),ve.push(()=>Fe(t,"show",r)),{c(){J(e.$$.fragment),n=v(),J(t.$$.fragment)},m(i,f){z(e,i,f),g(i,n,f),z(t,i,f),o=!0},p(i,[f]){const m={};f&2&&(m.data=i[1]),f&33&&(m.$$scope={dirty:f,ctx:i}),e.$set(m);const O={};f&2&&(O.gj=i[1]),!l&&f&1&&(l=!0,O.show=i[0],Ge(()=>l=!1)),t.$set(O)},i(i){o||(w(e.$$.fragment,i),w(t.$$.fragment,i),o=!0)},o(i){L(e.$$.fragment,i),L(t.$$.fragment,i),o=!1},d(i){i&&h(n),P(e,i),P(t,i)}}}function zn(s,e,n){let t,l,o;U(s,$t,i=>n(2,l=i)),U(s,ae,i=>n(3,o=i));let r=!0;function u(i){r=i,n(0,r)}return s.$$.update=()=>{s.$$.dirty&12&&n(1,t=o&&l?JSON.parse(o.debugMovementsFromLaneGeojson(l.properties.road,l.properties.index)):Ne())},[r,t,l,o,u]}class Pn extends se{constructor(e){super(),re(this,e,zn,Jn,x,{})}}function Rn(s){let e,n,t,l,o,r;return{c(){e=c("div"),n=c("label"),t=c("input"),l=R(`
    Clockwise ordering of roads`),q(t,"type","checkbox")},m(u,i){g(u,e,i),a(e,n),a(n,t),t.checked=s[0],a(n,l),o||(r=oe(t,"change",s[5]),o=!0)},p(u,[i]){i&1&&(t.checked=u[0])},i:V,o:V,d(u){u&&h(e),o=!1,r()}}}function En(s,e,n){let t,l,o;U(s,Wt,f=>n(2,t=f)),U(s,Jt,f=>n(3,l=f)),U(s,ae,f=>n(4,o=f));let r=[],u=!1;function i(){u=this.checked,n(0,u)}return s.$$.update=()=>{if(s.$$.dirty&31){for(let f of r)f.remove();if(n(1,r=[]),u&&l){let f=JSON.parse(o.debugClockwiseOrderingForIntersectionGeojson(l.properties.id));for(let m of f.features)r.push(new Ut.Popup({closeButton:!1,closeOnClick:!1,focusAfterOpen:!1}).setLngLat(m.geometry.coordinates).setHTML(m.properties.label).addTo(t))}}},[u,r,t,l,o,i]}class An extends se{constructor(e){super(),re(this,e,En,Rn,x,{})}}function Dn(s){let e,n;const t=[Re("debug-ids"),{layout:{"text-field":["get","id"],visibility:s[0]?"visible":"none"}},{paint:{"text-halo-color":["case",["==",["get","type"],"intersection"],"red","cyan"],"text-halo-width":3}}];let l={};for(let o=0;o<t.length;o+=1)l=Ee(l,t[o]);return e=new an({props:l}),{c(){J(e.$$.fragment)},m(o,r){z(e,o,r),n=!0},p(o,r){const u=r&1?Ue(t,[t[0],{layout:{"text-field":["get","id"],visibility:o[0]?"visible":"none"}},t[2]]):{};e.$set(u)},i(o){n||(w(e.$$.fragment,o),n=!0)},o(o){L(e.$$.fragment,o),n=!1},d(o){P(e,o)}}}function Hn(s){let e,n,t,l,o;e=new We({props:{data:s[1],generateId:!0,$$slots:{default:[Dn]},$$scope:{ctx:s}}});function r(i){s[3](i)}let u={gj:s[1],name:"Debug IDs",downloadable:!1};return s[0]!==void 0&&(u.show=s[0]),t=new Ye({props:u}),ve.push(()=>Fe(t,"show",r)),{c(){J(e.$$.fragment),n=v(),J(t.$$.fragment)},m(i,f){z(e,i,f),g(i,n,f),z(t,i,f),o=!0},p(i,[f]){const m={};f&2&&(m.data=i[1]),f&17&&(m.$$scope={dirty:f,ctx:i}),e.$set(m);const O={};f&2&&(O.gj=i[1]),!l&&f&1&&(l=!0,O.show=i[0],Ge(()=>l=!1)),t.$set(O)},i(i){o||(w(e.$$.fragment,i),w(t.$$.fragment,i),o=!0)},o(i){L(e.$$.fragment,i),L(t.$$.fragment,i),o=!1},d(i){i&&h(n),P(e,i),P(t,i)}}}function Fn(s,e,n){let t,l;U(s,ae,u=>n(2,l=u));let o=!1;function r(u){o=u,n(0,o)}return s.$$.update=()=>{s.$$.dirty&4&&n(1,t=l?JSON.parse(l.toGeojsonPlain()):Ne())},[o,t,l,r]}class Gn extends se{constructor(e){super(),re(this,e,Fn,Hn,x,{})}}function dt(s,e,n){const t=s.slice();return t[5]=e[n],t}function _t(s){const e=s.slice(),n=JSON.parse(e[0].crossing);return e[8]=n,e}function $n(s){let e,n,t,l=s[8].kind+"",o,r,u=s[8].has_island&&Wn();return{c(){e=c("p"),n=c("u"),n.textContent="Crossing",t=R(": "),o=R(l),r=v(),u&&u.c()},m(i,f){g(i,e,f),a(e,n),a(e,t),a(e,o),a(e,r),u&&u.m(e,null)},p:V,d(i){i&&h(e),u&&u.d()}}}function Wn(s){let e;return{c(){e=R("(with an island)")},m(n,t){g(n,e,t)},d(n){n&&h(e)}}}function gt(s){let e,n=s[5]+"",t,l;return{c(){e=c("a"),t=R(n),l=R(","),q(e,"href","https://www.openstreetmap.org/node/"+s[5]),q(e,"target","_blank")},m(o,r){g(o,e,r),a(e,t),g(o,l,r)},p:V,d(o){o&&(h(e),h(l))}}}function Un(s){let e,n,t,l,o,r=s[0].intersection_kind+"",u,i,f,m,O,S=s[0].control+"",C,M,y,j,I,k=s[0].movements+"",N,A,Q,D,G,W,H,X,K,_,ee,$=s[0].crossing&&$n(_t(s)),te=ge(JSON.parse(s[0].osm_node_ids)),F=[];for(let d=0;d<te.length;d+=1)F[d]=gt(dt(s,te,d));return{c(){e=c("h2"),e.textContent=`Intersection #${s[0].id}`,n=v(),t=c("p"),l=c("u"),l.textContent="Kind",o=R(": "),u=R(r),i=v(),f=c("p"),m=c("u"),m.textContent="Control",O=R(": "),C=R(S),M=v(),y=c("p"),j=c("u"),j.textContent="Movements",I=R(": "),N=R(k),A=v(),$&&$.c(),Q=v(),D=c("p"),G=c("u"),G.textContent="OSM nodes",W=R(`:
  `);for(let d=0;d<F.length;d+=1)F[d].c();H=v(),X=c("div"),K=c("button"),K.textContent="Collapse intersection",q(K,"type","button")},m(d,b){g(d,e,b),g(d,n,b),g(d,t,b),a(t,l),a(t,o),a(t,u),g(d,i,b),g(d,f,b),a(f,m),a(f,O),a(f,C),g(d,M,b),g(d,y,b),a(y,j),a(y,I),a(y,N),g(d,A,b),$&&$.m(d,b),g(d,Q,b),g(d,D,b),a(D,G),a(D,W);for(let B=0;B<F.length;B+=1)F[B]&&F[B].m(D,null);g(d,H,b),g(d,X,b),a(X,K),_||(ee=oe(K,"click",s[1]),_=!0)},p(d,[b]){if(d[0].crossing&&$.p(_t(d),b),b&1){te=ge(JSON.parse(d[0].osm_node_ids));let B;for(B=0;B<te.length;B+=1){const ce=dt(d,te,B);F[B]?F[B].p(ce,b):(F[B]=gt(ce),F[B].c(),F[B].m(D,null))}for(;B<F.length;B+=1)F[B].d(1);F.length=te.length}},i:V,o:V,d(d){d&&(h(e),h(n),h(t),h(i),h(f),h(M),h(y),h(A),h(Q),h(D),h(H),h(X)),$&&$.d(d),$e(F,d),_=!1,ee()}}}function qn(s,e,n){let t;U(s,ae,i=>n(4,t=i));let{data:l}=e,{close:o}=e,r=l.properties;function u(){t.collapseIntersection(r.id),ae.set(t),o()}return s.$$set=i=>{"data"in i&&n(2,l=i.data),"close"in i&&n(3,o=i.close)},[r,u,l,o]}class Vn extends se{constructor(e){super(),re(this,e,qn,Un,x,{data:2,close:3})}}function ht(s,e,n){const t=s.slice();return t[12]=e[n],t}function bt(s,e,n){const t=s.slice();return t[15]=e[n][0],t[16]=e[n][1],t}function Kn(s){let e,n,t,l;return{c(){e=c("details"),n=c("summary"),n.textContent="Full Muv JSON",t=v(),l=c("pre"),l.textContent=`${JSON.stringify(JSON.parse(s[0].muv),null,"  ")}`},m(o,r){g(o,e,r),a(e,n),a(e,t),a(e,l)},p:V,d(o){o&&h(e)}}}function kt(s){let e,n,t;return{c(){e=c("tr"),n=c("td"),n.textContent=`${s[15]}`,t=c("td"),t.textContent=`${s[16]}`,q(n,"class","svelte-860yh4"),q(t,"class","svelte-860yh4")},m(l,o){g(l,e,o),a(e,n),a(e,t)},p:V,d(l){l&&h(e)}}}function yt(s){let e,n,t=s[12]+"",l,o,r,u,i,f,m,O=ge(Object.entries(JSON.parse(s[1].getOsmTagsForWay(BigInt(s[12]))))),S=[];for(let C=0;C<O.length;C+=1)S[C]=kt(bt(s,O,C));return{c(){e=c("p"),n=c("a"),l=R(t),o=v(),r=c("details"),u=c("summary"),u.textContent="See OSM tags",i=v(),f=c("table"),m=c("tbody");for(let C=0;C<S.length;C+=1)S[C].c();q(n,"href","https://www.openstreetmap.org/way/"+s[12]),q(n,"target","_blank")},m(C,M){g(C,e,M),a(e,n),a(n,l),g(C,o,M),g(C,r,M),a(r,u),a(r,i),a(r,f),a(f,m);for(let y=0;y<S.length;y+=1)S[y]&&S[y].m(m,null)},p(C,M){if(M&3){O=ge(Object.entries(JSON.parse(C[1].getOsmTagsForWay(BigInt(C[12])))));let y;for(y=0;y<O.length;y+=1){const j=bt(C,O,y);S[y]?S[y].p(j,M):(S[y]=kt(j),S[y].c(),S[y].m(m,null))}for(;y<S.length;y+=1)S[y].d(1);S.length=O.length}},d(C){C&&(h(e),h(o),h(r)),$e(S,C)}}}function Zn(s){let e,n,t,l,o,r=s[0].type+"",u,i,f,m,O,S=s[0].direction+"",C,M,y,j,I,k=s[0].width+"",N,A,Q,D,G,W,H=s[0].speed_limit+"",X,K,_,ee,$,te=s[0].allowed_turns+"",F,d,b,B,ce,he=s[0].layer+"",ne,le,ie,pe,we,be,Ce,_e,ue,fe,Ae,ke,p,me,de,xe,Be,Ve,Oe,Je,et,ze,Ke,tt,ye=s[0].muv&&Kn(s),Le=ge(JSON.parse(s[0].osm_way_ids)),Y=[];for(let T=0;T<Le.length;T+=1)Y[T]=yt(ht(s,Le,T));return{c(){e=c("h2"),e.textContent=`Lane ${s[0].index} of Road ${s[0].road}`,n=v(),t=c("p"),l=c("u"),l.textContent="Type",o=R(": "),u=R(r),i=v(),f=c("p"),m=c("u"),m.textContent="Direction",O=R(": "),C=R(S),M=v(),y=c("p"),j=c("u"),j.textContent="Width",I=R(": "),N=R(k),A=R("m"),Q=v(),D=c("p"),G=c("u"),G.textContent="Speed limit",W=R(": "),X=R(H),K=v(),_=c("p"),ee=c("u"),ee.textContent="Allowed turns",$=R(": "),F=R(te),d=v(),b=c("p"),B=c("u"),B.textContent="Layer",ce=R(": "),ne=R(he),le=v(),ye&&ye.c(),ie=v(),pe=c("hr"),we=v(),be=c("p"),be.innerHTML="<u>OSM ways:</u>",Ce=v();for(let T=0;T<Y.length;T+=1)Y[T].c();_e=v(),ue=c("div"),fe=c("button"),fe.textContent="Collapse short road",Ae=v(),ke=c("button"),ke.textContent="Zip side-path",p=v(),me=c("div"),de=c("button"),de.textContent="Find block on left",xe=v(),Be=c("button"),Be.textContent="Find block on right",Ve=v(),Oe=c("div"),Je=c("button"),Je.textContent="Trace sidewalks on left",et=v(),ze=c("button"),ze.textContent="Trace sidewalks on right",q(fe,"type","button"),q(ke,"type","button"),q(de,"type","button"),q(Be,"type","button"),q(Je,"type","button"),q(ze,"type","button")},m(T,E){g(T,e,E),g(T,n,E),g(T,t,E),a(t,l),a(t,o),a(t,u),g(T,i,E),g(T,f,E),a(f,m),a(f,O),a(f,C),g(T,M,E),g(T,y,E),a(y,j),a(y,I),a(y,N),a(y,A),g(T,Q,E),g(T,D,E),a(D,G),a(D,W),a(D,X),g(T,K,E),g(T,_,E),a(_,ee),a(_,$),a(_,F),g(T,d,E),g(T,b,E),a(b,B),a(b,ce),a(b,ne),g(T,le,E),ye&&ye.m(T,E),g(T,ie,E),g(T,pe,E),g(T,we,E),g(T,be,E),g(T,Ce,E);for(let Z=0;Z<Y.length;Z+=1)Y[Z]&&Y[Z].m(T,E);g(T,_e,E),g(T,ue,E),a(ue,fe),a(ue,Ae),a(ue,ke),g(T,p,E),g(T,me,E),a(me,de),a(me,xe),a(me,Be),g(T,Ve,E),g(T,Oe,E),a(Oe,Je),a(Oe,et),a(Oe,ze),Ke||(tt=[oe(fe,"click",s[2]),oe(ke,"click",s[3]),oe(de,"click",s[7]),oe(Be,"click",s[8]),oe(Je,"click",s[9]),oe(ze,"click",s[10])],Ke=!0)},p(T,[E]){if(T[0].muv&&ye.p(T,E),E&3){Le=ge(JSON.parse(T[0].osm_way_ids));let Z;for(Z=0;Z<Le.length;Z+=1){const nt=ht(T,Le,Z);Y[Z]?Y[Z].p(nt,E):(Y[Z]=yt(nt),Y[Z].c(),Y[Z].m(_e.parentNode,_e))}for(;Z<Y.length;Z+=1)Y[Z].d(1);Y.length=Le.length}},i:V,o:V,d(T){T&&(h(e),h(n),h(t),h(i),h(f),h(M),h(y),h(Q),h(D),h(K),h(_),h(d),h(b),h(le),h(ie),h(pe),h(we),h(be),h(Ce),h(_e),h(ue),h(p),h(me),h(Ve),h(Oe)),ye&&ye.d(T),$e(Y,T),Ke=!1,Bt(tt)}}}function Qn(s,e,n){let t;U(s,ae,y=>n(11,t=y));let{data:l}=e,{close:o}=e,r=l.properties,u=t;function i(){t.collapseShortRoad(r.road),ae.set(t),o()}function f(){t.zipSidepath(r.road),ae.set(t),o()}function m(y,j){try{Pe.set(JSON.parse(t.findBlock(r.road,y,j))),Ze.set(j),o()}catch(I){window.alert(I)}}const O=()=>m(!0,!1),S=()=>m(!1,!1),C=()=>m(!0,!0),M=()=>m(!1,!0);return s.$$set=y=>{"data"in y&&n(5,l=y.data),"close"in y&&n(6,o=y.close)},[r,u,i,f,m,l,o,O,S,C,M]}class Xn extends se{constructor(e){super(),re(this,e,Qn,Zn,x,{data:5,close:6})}}function vt(s){let e,n;return e=new Qt({}),{c(){J(e.$$.fragment)},m(t,l){z(e,t,l),n=!0},i(t){n||(w(e.$$.fragment,t),n=!0)},o(t){L(e.$$.fragment,t),n=!1},d(t){P(e,t)}}}function Yn(s){let e,n,t,l,o,r,u,i,f,m,O,S,C,M,y,j,I;l=new Vt({});let k=s[2]&&vt();return{c(){e=c("div"),n=c("h1"),n.textContent="osm2streets Street Explorer",t=v(),J(l.$$.fragment),o=v(),r=c("p"),r.innerHTML=`Understanding OSM streets &amp; intersections with
      <a href="https://github.com/a-b-street/osm2streets" target="_blank">osm2streets</a>
      and <a href="https://gitlab.com/LeLuxNet/Muv" target="_blank">Muv</a>.`,u=v(),i=c("hr"),f=v(),k&&k.c(),m=v(),O=c("br"),S=v(),C=c("details"),M=c("summary"),M.textContent="Layers",y=v(),j=c("div"),C.open=!0,q(C,"class","svelte-1n0zlav"),q(e,"slot","left")},m(N,A){g(N,e,A),a(e,n),a(e,t),z(l,e,null),a(e,o),a(e,r),a(e,u),a(e,i),a(e,f),k&&k.m(e,null),a(e,m),a(e,O),a(e,S),a(e,C),a(C,M),a(C,y),a(C,j),s[4](j),I=!0},p(N,A){N[2]?k?A&4&&w(k,1):(k=vt(),k.c(),w(k,1),k.m(e,m)):k&&(Ie(),L(k,1,1,()=>{k=null}),je())},i(N){I||(w(l.$$.fragment,N),w(k),I=!0)},o(N){L(l.$$.fragment,N),L(k),I=!1},d(N){N&&h(e),P(l),k&&k.d(),s[4](null)}}}function wt(s){let e,n;return e=new Vn({props:{data:s[5],close:s[6]}}),{c(){J(e.$$.fragment)},m(t,l){z(e,t,l),n=!0},p(t,l){const o={};l&32&&(o.data=t[5]),l&64&&(o.close=t[6]),e.$set(o)},i(t){n||(w(e.$$.fragment,t),n=!0)},o(t){L(e.$$.fragment,t),n=!1},d(t){P(e,t)}}}function xn(s){let e=s[5],n,t,l=wt(s);return{c(){l.c(),n=He()},m(o,r){l.m(o,r),g(o,n,r),t=!0},p(o,r){r&32&&x(e,e=o[5])?(Ie(),L(l,1,1,V),je(),l=wt(o),l.c(),w(l,1),l.m(n.parentNode,n)):l.p(o,r)},i(o){t||(w(l),t=!0)},o(o){L(l),t=!1},d(o){o&&h(n),l.d(o)}}}function el(s){let e,n;return e=new qe({props:{openOn:"click",popupClass:"popup",$$slots:{default:[xn,({data:t,close:l})=>({5:t,6:l}),({data:t,close:l})=>(t?32:0)|(l?64:0)]},$$scope:{ctx:s}}}),{c(){J(e.$$.fragment)},m(t,l){z(e,t,l),n=!0},p(t,l){const o={};l&224&&(o.$$scope={dirty:l,ctx:t}),e.$set(o)},i(t){n||(w(e.$$.fragment,t),n=!0)},o(t){L(e.$$.fragment,t),n=!1},d(t){P(e,t)}}}function Ct(s){let e,n;return e=new Xn({props:{data:s[5],close:s[6]}}),{c(){J(e.$$.fragment)},m(t,l){z(e,t,l),n=!0},p(t,l){const o={};l&32&&(o.data=t[5]),l&64&&(o.close=t[6]),e.$set(o)},i(t){n||(w(e.$$.fragment,t),n=!0)},o(t){L(e.$$.fragment,t),n=!1},d(t){P(e,t)}}}function tl(s){let e=s[5],n,t,l=Ct(s);return{c(){l.c(),n=He()},m(o,r){l.m(o,r),g(o,n,r),t=!0},p(o,r){r&32&&x(e,e=o[5])?(Ie(),L(l,1,1,V),je(),l=Ct(o),l.c(),w(l,1),l.m(n.parentNode,n)):l.p(o,r)},i(o){t||(w(l),t=!0)},o(o){L(l),t=!1},d(o){o&&h(n),l.d(o)}}}function nl(s){let e,n;return e=new qe({props:{openOn:"click",popupClass:"popup",$$slots:{default:[tl,({data:t,close:l})=>({5:t,6:l}),({data:t,close:l})=>(t?32:0)|(l?64:0)]},$$scope:{ctx:s}}}),{c(){J(e.$$.fragment)},m(t,l){z(e,t,l),n=!0},p(t,l){const o={};l&224&&(o.$$scope={dirty:l,ctx:t}),e.$set(o)},i(t){n||(w(e.$$.fragment,t),n=!0)},o(t){L(e.$$.fragment,t),n=!1},d(t){P(e,t)}}}function ll(s){let e,n,t,l,o,r,u,i,f,m,O,S,C,M,y,j,I,k,N,A,Q,D,G,W,H,X,K,_,ee,$,te,F,d;return n=new Xt({}),l=new Yt({props:{hoverCursor:"pointer",$$slots:{default:[el]},$$scope:{ctx:s}}}),r=new xt({}),i=new en({props:{hoverCursor:"pointer",$$slots:{default:[nl]},$$scope:{ctx:s}}}),m=new tn({}),M=new Sn({}),k=new Pn({}),A=new An({}),D=new Nn({}),W=new Gn({}),_=new gn({}),$=new kn({}),F=new nn({}),{c(){e=c("div"),J(n.$$.fragment),t=v(),J(l.$$.fragment),o=v(),J(r.$$.fragment),u=v(),J(i.$$.fragment),f=v(),J(m.$$.fragment),O=v(),S=c("hr"),C=v(),J(M.$$.fragment),y=v(),j=c("hr"),I=v(),J(k.$$.fragment),N=v(),J(A.$$.fragment),Q=v(),J(D.$$.fragment),G=v(),J(W.$$.fragment),H=v(),X=c("hr"),K=v(),J(_.$$.fragment),ee=v(),J($.$$.fragment),te=v(),J(F.$$.fragment)},m(b,B){g(b,e,B),z(n,e,null),a(e,t),z(l,e,null),a(e,o),z(r,e,null),a(e,u),z(i,e,null),a(e,f),z(m,e,null),a(e,O),a(e,S),a(e,C),z(M,e,null),a(e,y),a(e,j),a(e,I),z(k,e,null),a(e,N),z(A,e,null),a(e,Q),z(D,e,null),a(e,G),z(W,e,null),a(e,H),a(e,X),a(e,K),z(_,e,null),a(e,ee),z($,e,null),a(e,te),z(F,e,null),s[3](e),d=!0},p(b,B){const ce={};B&128&&(ce.$$scope={dirty:B,ctx:b}),l.$set(ce);const he={};B&128&&(he.$$scope={dirty:B,ctx:b}),i.$set(he)},i(b){d||(w(n.$$.fragment,b),w(l.$$.fragment,b),w(r.$$.fragment,b),w(i.$$.fragment,b),w(m.$$.fragment,b),w(M.$$.fragment,b),w(k.$$.fragment,b),w(A.$$.fragment,b),w(D.$$.fragment,b),w(W.$$.fragment,b),w(_.$$.fragment,b),w($.$$.fragment,b),w(F.$$.fragment,b),d=!0)},o(b){L(n.$$.fragment,b),L(l.$$.fragment,b),L(r.$$.fragment,b),L(i.$$.fragment,b),L(m.$$.fragment,b),L(M.$$.fragment,b),L(k.$$.fragment,b),L(A.$$.fragment,b),L(D.$$.fragment,b),L(W.$$.fragment,b),L(_.$$.fragment,b),L($.$$.fragment,b),L(F.$$.fragment,b),d=!1},d(b){b&&h(e),P(n),P(l),P(r),P(i),P(m),P(M),P(k),P(A),P(D),P(W),P(_),P($),P(F),s[3](null)}}}function ol(s){let e,n,t;return n=new Kt({props:{$$slots:{default:[ll]},$$scope:{ctx:s}}}),{c(){e=c("div"),J(n.$$.fragment),q(e,"slot","main")},m(l,o){g(l,e,o),z(n,e,null),t=!0},p(l,o){const r={};o&129&&(r.$$scope={dirty:o,ctx:l}),n.$set(r)},i(l){t||(w(n.$$.fragment,l),t=!0)},o(l){L(n.$$.fragment,l),t=!1},d(l){l&&h(e),P(n)}}}function sl(s){let e,n;return e=new qt({props:{$$slots:{main:[ol],left:[Yn]},$$scope:{ctx:s}}}),{c(){J(e.$$.fragment)},m(t,l){z(e,t,l),n=!0},p(t,[l]){const o={};l&135&&(o.$$scope={dirty:l,ctx:t}),e.$set(o)},i(t){n||(w(e.$$.fragment,t),n=!0)},o(t){L(e.$$.fragment,t),n=!1},d(t){P(e,t)}}}function rl(s,e,n){let t=!1;Ot(async()=>{await Zt(),n(2,t=!0)});let l=null,o;function r(i){ve[i?"unshift":"push"](()=>{l=i,n(0,l)})}function u(i){ve[i?"unshift":"push"](()=>{o=i,n(1,o),n(0,l)})}return s.$$.update=()=>{s.$$.dirty&3&&l&&o&&(n(1,o.innerHTML="",o),o.appendChild(l))},[l,o,t,r,u]}class il extends se{constructor(e){super(),re(this,e,rl,sl,x,{})}}new il({target:document.getElementById("app")});